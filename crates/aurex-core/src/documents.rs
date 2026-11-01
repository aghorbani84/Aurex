//! Opaque document navigation. A content URI is never interpreted as a local path.
//!
//! Native SAF must still verify the OS grant. This model only authorizes identities
//! actually returned by that provider and refuses stale asynchronous responses.
use crate::{Error, ErrorKind, Result};
use std::collections::BTreeMap;

const MAX_KNOWN: usize = 4096;
const MAX_HISTORY: usize = 100;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentRequest {
    pub generation: u64,
    pub revision: u64,
    pub folder: String,
}

#[derive(Clone, Debug)]
pub struct DocumentSession {
    root: String,
    authority: String,
    known: BTreeMap<String, bool>,
    history: Vec<String>,
    revision: u64,
    generation: u64,
}

fn authority(uri: &str) -> Result<&str> {
    if uri.len() > 16 * 1024 || uri.chars().any(char::is_control) {
        return Err(Error::new(
            ErrorKind::InvalidLocation,
            "invalid document URI",
        ));
    }
    let rest = uri.strip_prefix("content://").ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidLocation,
            "only SAF content URIs are accepted",
        )
    })?;
    let (authority, path) = rest
        .split_once('/')
        .ok_or_else(|| Error::new(ErrorKind::InvalidLocation, "document URI has no identity"))?;
    if authority.is_empty()
        || path.is_empty()
        || uri.contains(['?', '#'])
        || !authority
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
    {
        return Err(Error::new(
            ErrorKind::InvalidLocation,
            "invalid content authority or identity",
        ));
    }
    Ok(authority)
}

impl DocumentSession {
    pub fn new(root: String) -> Result<Self> {
        Self::with_generation(root, 0)
    }
    pub fn with_generation(root: String, generation: u64) -> Result<Self> {
        let authority = authority(&root)?.to_owned();
        Ok(Self {
            known: BTreeMap::from([(root.clone(), true)]),
            history: vec![root.clone()],
            root,
            authority,
            revision: 0,
            generation,
        })
    }
    pub fn request(&self) -> DocumentRequest {
        DocumentRequest {
            generation: self.generation,
            revision: self.revision,
            folder: self.history.last().unwrap().clone(),
        }
    }
    pub fn root(&self) -> &str {
        &self.root
    }
    pub fn can_go_back(&self) -> bool {
        self.history.len() > 1
    }
    /// Register one bounded native page atomically; stale/foreign rows do not enter the allow-set.
    pub fn accept_page(
        &mut self,
        request: &DocumentRequest,
        rows: &[(String, bool)],
    ) -> Result<()> {
        if request != &self.request() {
            return Err(Error::new(
                ErrorKind::Changed,
                "discarded stale document response",
            ));
        }
        let mut additions = BTreeMap::new();
        for (uri, directory) in rows {
            if authority(uri)? != self.authority {
                return Err(Error::new(
                    ErrorKind::Permission,
                    "foreign document authority",
                ));
            }
            if let Some(previous) = self.known.get(uri) {
                if previous != directory {
                    return Err(Error::new(
                        ErrorKind::Changed,
                        "document kind changed; reselect folder",
                    ));
                }
            }
            if additions.insert(uri.clone(), *directory).is_some() {
                return Err(Error::new(
                    ErrorKind::Changed,
                    "duplicate document identity",
                ));
            }
        }
        let new_count = additions
            .keys()
            .filter(|uri| !self.known.contains_key(*uri))
            .count();
        if self.known.len() + new_count > MAX_KNOWN {
            return Err(Error::new(
                ErrorKind::Limit,
                "document session limit; reselect folder",
            ));
        }
        self.known.extend(additions);
        Ok(())
    }
    pub fn authorize_preview(&self, uri: &str) -> Result<()> {
        if self.known.get(uri) == Some(&false) {
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::Permission,
                "preview requires a discovered regular document",
            ))
        }
    }
    pub fn enter(&mut self, uri: &str) -> Result<()> {
        if self.known.get(uri) != Some(&true) {
            return Err(Error::new(
                ErrorKind::Permission,
                "directory was not discovered in this grant",
            ));
        }
        if self.history.len() >= MAX_HISTORY {
            return Err(Error::new(ErrorKind::Limit, "document history limit"));
        }
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| Error::new(ErrorKind::Limit, "revision exhausted"))?;
        self.history.push(uri.to_owned());
        Ok(())
    }
    pub fn back(&mut self) {
        if self.can_go_back() {
            self.history.pop();
            self.revision += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const ROOT: &str = "content://provider/tree/opaque%3Aid/document/opaque%3Aid";
    fn session() -> DocumentSession {
        DocumentSession::new(ROOT.into()).unwrap()
    }
    #[test]
    fn opaque_identity_is_not_a_filesystem_path() {
        let mut s = session();
        let request = s.request();
        let child = "content://provider/tree/a/document/%2F..%2Fnot-a-path".to_owned();
        s.accept_page(&request, &[(child.clone(), true)]).unwrap();
        s.enter(&child).unwrap();
        assert_eq!(s.request().folder, child);
        s.back();
        assert_eq!(s.request().folder, ROOT);
    }
    #[test]
    fn forged_uri_and_file_navigation_are_rejected() {
        let mut s = session();
        assert!(s.enter("content://provider/document/forged").is_err());
        assert!(s.authorize_preview(ROOT).is_err());
        let file = "content://provider/document/file".to_owned();
        s.accept_page(&s.request(), &[(file.clone(), false)])
            .unwrap();
        assert!(s.enter(&file).is_err());
        s.authorize_preview(&file).unwrap();
    }
    #[test]
    fn foreign_page_is_atomic() {
        let mut s = session();
        let file = "content://provider/document/file".to_owned();
        assert!(
            s.accept_page(
                &s.request(),
                &[
                    (file.clone(), false),
                    ("content://foreign/document/x".into(), false)
                ]
            )
            .is_err()
        );
        assert!(s.authorize_preview(&file).is_err());
    }
    #[test]
    fn stale_pages_cannot_authorize_documents() {
        let mut s = session();
        let old = s.request();
        let dir = "content://provider/document/dir".to_owned();
        s.accept_page(&old, &[(dir.clone(), true)]).unwrap();
        s.enter(&dir).unwrap();
        let file = "content://provider/document/stale".to_owned();
        assert_eq!(
            s.accept_page(&old, &[(file.clone(), false)])
                .unwrap_err()
                .kind,
            ErrorKind::Changed
        );
        assert!(s.authorize_preview(&file).is_err());
    }
    #[test]
    fn regranting_the_same_uri_invalidates_old_work() {
        let old = DocumentSession::with_generation(ROOT.into(), 1)
            .unwrap()
            .request();
        let mut current = DocumentSession::with_generation(ROOT.into(), 2).unwrap();
        assert_eq!(
            current.accept_page(&old, &[]).unwrap_err().kind,
            ErrorKind::Changed
        );
    }
    #[test]
    fn malformed_and_non_content_grants_are_rejected() {
        for uri in [
            "file:///tmp",
            "content://evil@provider/x",
            "content://provider/",
            "content://provider/x?query",
            "content://provider/x\0",
            "content://provider/x#fragment",
        ] {
            assert!(DocumentSession::new(uri.into()).is_err(), "{uri:?}");
        }
    }
    #[test]
    fn bounded_sessions_refuse_instead_of_silently_evicting_authority() {
        let mut s = session();
        let rows: Vec<_> = (0..MAX_KNOWN)
            .map(|i| (format!("content://provider/document/{i}"), false))
            .collect();
        assert_eq!(
            s.accept_page(&s.request(), &rows).unwrap_err().kind,
            ErrorKind::Limit
        );
        assert!(s.authorize_preview(&rows[0].0).is_err());
    }
}
