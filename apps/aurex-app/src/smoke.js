// Executed only by the explicit, feature-gated native fixture harness.
const wait = async predicate => {
    for (let i = 0; i < 150; i++) {
        if (predicate()) return;
        await new Promise(resolve => setTimeout(resolve, 100));
    }
    throw new Error("AUREX native fixture assertion timed out");
};
const assert = condition => { if (!condition) throw new Error("AUREX native fixture assertion failed"); };
const pane = () => document.querySelector(".pane");
const file = name => Array.from(pane().querySelectorAll("button.file-name")).find(e => e.textContent.trim().endsWith(name));
await wait(() => document.querySelectorAll(".pane").length === 2 && file("aurex-smoke.txt"));
assert(document.querySelector("h1").textContent === "A place for everything.");
const copy = Array.from(document.querySelectorAll(".actionbar button")).find(b => b.textContent.includes("Copy to other pane"));
assert(copy && copy.disabled);
file("nested").click();
await wait(() => file("inside.txt"));
pane().querySelector('button[aria-label="Back"]').click();
await wait(() => file("aurex-smoke.txt"));
let filter = pane().querySelector('input[aria-label="Filter directory by filename"]');
filter.value = "no-such-aurex-fixture";
filter.dispatchEvent(new Event("input", {bubbles:true}));
await wait(() => pane().querySelectorAll(".file-row").length === 0 && pane().textContent.includes("Nothing here yet"));
filter = pane().querySelector('input[aria-label="Filter directory by filename"]');
filter.value = "";
filter.dispatchEvent(new Event("input", {bubbles:true}));
await wait(() => file("aurex-smoke.txt"));
pane().querySelector('button[aria-label="Duplicate current tab"]').click();
await wait(() => pane().querySelectorAll(".tabs .tab").length === 2);
pane().querySelector('button[aria-label="Close current tab"]').click();
await wait(() => pane().querySelectorAll(".tabs .tab").length === 1 && file("aurex-smoke.txt"));
file("aurex-smoke.txt").click();
await wait(() => document.querySelector(".preview-modal pre"));
assert(document.querySelector(".preview-modal pre").textContent === "AUREX native runtime verification\n");
document.querySelector(".preview-modal button").click();
await wait(() => !document.querySelector(".preview-modal"));
return true;
