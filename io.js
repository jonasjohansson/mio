const { shell, dialog } = require("electron").remote;
const EventEmitter = require("event-emitter-es6");

const config = require("./config");

class IO extends EventEmitter {
  constructor(data) {
    super();

    this.view = elDiv(document.body);
    this.view.classList.add("view");

    // serial
    var div = elDiv(this.view);
    // this.ids = el("select", null, "pin", div);
    this.ids = el("input", "text", "id", div);
    this.input = el("input", "number", "in", div);
    this.inMin = el("input", "number", "min", div);
    this.inMax = el("input", "number", "max", div);
    this.smooth = el("input", "range", "smooth", div);
    this.output = el("input", "number", "out", div);
    this.id = data.index;
    this.input.disabled = true;
    this.input.setAttribute("data-value", 0);
    this.output.disabled = true;

    this.smooth.parentNode.style.display = "none";
    this.inMin.parentNode.style.display = "none";
    this.inMax.parentNode.style.display = "none";

    for (var input of [this.input, this.inMin, this.inMax]) {
      input.addEventListener("change", e => {
        this.update();
      });
    }

    // keys
    var div = elDiv(this.view);
    this.keys = el("select", null, "key", div);
    this.keyMod = el("select", null, "mod", div);
    this.keySend = el("input", "checkbox", "send key", div);
    this.keyHold = el("input", "checkbox", "long press", div);
    this.keyPressed = false;

    // midi
    this.midiControl = el("input", "number", "midi control", div);
    this.midiChannel = el("input", "number", "midi channel", div);
    this.midiSend = el("input", "checkbox", "send midi", div);

    this.midiSend.parentNode.style.display = "none";
    this.midiControl.parentNode.style.display = "none";
    this.midiChannel.parentNode.style.display = "none";

    this.setKeys();
    this.setMods();

    this.ids.value = data.id;
    this.inMin = 0;
    this.inMax = 1023;
    this.outMin = 0;
    this.outMax = 127;
    this.keys.value = data.key || "a";
    this.keySend.checked = data.keySend || false;
    this.keyHold.checked = data.keyHold || false;
    this.midiSend.checked = data.midiSend || true;
    this.midiControl.value = data.midiControl || 172;
    this.midiChannel.value = data.midiChannel || 1;
  }

  update(val) {
    var a = Number(this.input.getAttribute("data-temp")) || 0;

    var b = lerp(a, val, this.smooth.value);

    var mapped = b.map(this.inMin, this.inMax, this.outMin, this.outMax);

    mapped = Math.ceil(mapped);

    this.input.setAttribute("data-temp", b);
    this.input.value = val;
    this.output.value = Math.round(mapped);
  }

  setKeys() {
    for (const key of config.get("keys")) {
      let option = document.createElement("option");
      option.value = key;
      option.textContent = key;
      this.keys.appendChild(option);
    }
  }

  setMods() {
    for (const mod of config.get("mods")) {
      let option = document.createElement("option");
      option.value = mod;
      option.textContent = mod;
      this.keyMod.appendChild(option);
    }
  }

  calibrate() {
    var min = 9999;
    var max = 0;
    var val = Number(this.input.value);
    if (val > max) this.inMax = val;
    if (val < min) this.inMin = val;
  }
}

function el(nodeType, type = null, labelName = null, parent = null, value = 0) {
  var el = document.createElement(nodeType);
  el.type = type;
  if (type === "number") {
    el.value = value;
    el.setAttribute("inputmode", "numeric");
    el.setAttribute("min", 0);
  }
  if (type === "range") {
    el.setAttribute("min", 0.001);
    el.setAttribute("max", 1.0);
    el.setAttribute("step", 0.001);
    el.value = 1;
  }
  if (parent !== null) {
    var div = document.createElement("div");
    if (labelName !== null) {
      label = document.createElement("label");
      label.textContent = labelName;
      div.appendChild(label);
      // set unique id
      var id = makeid(4);
      label.setAttribute("for", id);
      el.id = id;
    }
    div.appendChild(el);
    parent.appendChild(div);
  }
  return el;
}

function elDiv(parent) {
  var el = document.createElement("div");
  parent.appendChild(el);
  return el;
}

function makeid(length) {
  var text = "";
  var possible =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

  for (var i = 0; i < length; i++) {
    text += possible.charAt(Math.floor(Math.random() * possible.length));
  }

  return text;
}

function lerp(v0, v1, t) {
  return v0 + t * (v1 - v0);
  // return v0 * (1 - t) + v1 * t;
}

function smoothstep(min, max, value) {
  var x = Math.max(0, Math.min(1, (value - min) / (max - min)));
  return x * x * (3 - 2 * x);
}

Number.prototype.map = function(in_min, in_max, out_min, out_max) {
  return ((this - in_min) * (out_max - out_min)) / (in_max - in_min) + out_min;
};
Number.prototype.clamp = function(min, max) {
  return Math.min(Math.max(this, min), max);
};
module.exports = IO;
