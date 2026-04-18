function omnipresentChanged(cb) {
    const els = document.getElementsByClassName("omnipresent");
    const add = cb.checked;
    for (let i = 0; i < els.length; i++) {
        if (add) {
            els[i].classList.add("hidden");
        } else {
            els[i].classList.remove("hidden");
        }
    }
}

function archVersionsOnLoad() {
    const checkboxes = document.querySelectorAll('input[type="checkbox"]');
    checkboxes.forEach(cb => cb.checked = false);
}