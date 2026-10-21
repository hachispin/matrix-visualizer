// Egui's canvas widgets cannot summon a mobile keyboard. Use a real input for a
// tapped number, leaving mouse and hardware-keyboard editing in egui.
let canvas;
let input;
let fields = new Map();
const committed = new Map();
let active;
let gesture;
let canvasRect;
let scaleX = 1;
let scaleY = 1;

function finish(cancelled = false) {
    if (!active) {
        return;
    }

    if (!cancelled) {
        committed.set(active.id, input.value);
    }

    active = undefined;
    input.hidden = true;
    input.blur();
}

function position(field) {
    const [x, y, width, height, left, top, right, bottom] = field.bounds;
    const viewport = window.visualViewport;
    const visibleTop = viewport?.offsetTop ?? 0;
    const visibleBottom = visibleTop + (viewport?.height ?? window.innerHeight);
    const fieldTop = canvasRect.top + y * scaleY;
    const editorTop = Math.max(visibleTop, Math.min(fieldTop, visibleBottom - height * scaleY));
    const clipTop = editorTop === fieldTop ? Math.max(0, top - y) * scaleY : 0;
    const clipBottom = editorTop === fieldTop ? Math.max(0, y + height - bottom) * scaleY : 0;

    Object.assign(input.style, {
        left: `${canvasRect.left + x * scaleX}px`,
        top: `${editorTop}px`,
        width: `${width * scaleX}px`,
        height: `${height * scaleY}px`,
        clipPath: `inset(${clipTop}px ${Math.max(0, x + width - right) * scaleX}px ${clipBottom}px ${Math.max(0, left - x) * scaleX}px)`,
    });
}

function field_at(event) {
    const x = (event.clientX - canvasRect.left) / scaleX;
    const y = (event.clientY - canvasRect.top) / scaleY;

    return [...fields.values()].find(({ bounds: [fx, fy, width, height, left, top, right, bottom] }) =>
        x >= Math.max(fx, left) && x <= Math.min(fx + width, right)
        && y >= Math.max(fy, top) && y <= Math.min(fy + height, bottom));
}

function initialize() {
    canvas = document.getElementById("canvas");
    input = document.createElement("input");
    input.type = "text";
    // Text mode retains minus signs and exponents on keyboards whose decimal
    // layout only offers digits and a decimal separator.
    input.inputMode = "text";
    input.enterKeyHint = "done";
    input.autocomplete = "off";
    input.autocapitalize = "off";
    input.spellcheck = false;
    input.hidden = true;

    Object.assign(input.style, {
        position: "fixed",
        zIndex: "1",
        boxSizing: "border-box",
        minWidth: "0",
        margin: "0",
        padding: "2px 4px",
        border: "1px solid #5794d0",
        borderRadius: "3px",
        outline: "none",
        background: "#fff",
        color: "#222",
        // At least 16px prevents Safari from zooming the page when focusing.
        font: "16px sans-serif",
        touchAction: "auto",
    });

    document.body.append(input);
    input.addEventListener("blur", () => finish());
    input.addEventListener("keydown", (event) => {
        if (event.key === "Enter" || event.key === "Escape" || event.key === "Tab") {
            event.preventDefault();
            finish(event.key === "Escape");
        }
    });

    input.addEventListener("input", () => {
        const normalized = input.value.replace(/\s/g, "").replaceAll("−", "-");
        const number = Number(normalized);

        if (/^[+-]?(inf(inity)?|nan)$/i.test(normalized)
            || (!Number.isNaN(number) && !Number.isFinite(Math.fround(number)))) {
            input.value = active.value;
        }
    });

    window.visualViewport?.addEventListener("resize", () => {
        if (active) {
            position(active);
        }
    });

    canvas.addEventListener("pointerdown", (event) => {
        finish();

        if (event.pointerType !== "touch" || !event.isPrimary) {
            gesture = undefined;

            return;
        }

        gesture = { id: event.pointerId, x: event.clientX, y: event.clientY, field: field_at(event) };
    }, true);

    canvas.addEventListener("pointermove", (event) => {
        if (gesture && (gesture.id !== event.pointerId
            || Math.hypot(event.clientX - gesture.x, event.clientY - gesture.y) > 8)) {
            gesture = undefined;
        }
    }, true);

    canvas.addEventListener("pointercancel", () => { gesture = undefined; }, true);
    canvas.addEventListener("pointerup", (event) => {
        const tap = gesture;
        gesture = undefined;

        if (!tap?.field || tap.id !== event.pointerId || field_at(event)?.id !== tap.field.id) {
            return;
        }

        active = { ...tap.field, seen: true };
        input.value = active.value;
        input.setAttribute("aria-label", active.label || "Number");
        position(active);
        input.hidden = false;
        // This must happen synchronously during the tap, especially on iOS.
        input.focus({ preventScroll: true });
        input.select();
    }, true);
}

export function begin_frame(width, height) {
    if (!input) {
        initialize();
    }

    canvasRect = canvas.getBoundingClientRect();
    scaleX = canvasRect.width / width;
    scaleY = canvasRect.height / height;
    fields = new Map();

    if (active) {
        active.seen = false;
    }
}

export function register_number(id, value, label, bounds) {
    const field = { id, value, label, bounds: Array.from(bounds) };
    fields.set(id, field);

    if (active?.id === id) {
        active = { ...field, seen: true };
        position(field);
    }
}

export function end_frame() {
    // Removed widgets must not leave an edit waiting for a future widget that
    // happens to reuse their egui id.
    committed.clear();

    if (active && !active.seen) {
        finish(true);
    }
}

export function is_editing(id) {
    return active?.id === id;
}

export function take_value(id) {
    const value = committed.get(id);
    committed.delete(id);

    return value;
}
