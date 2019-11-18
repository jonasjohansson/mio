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
    this.pins = el("select", null, "pin", div);
    this.input = el("input", "number", "in", div);
    this.inMin = el("input", "number", "min", div);
    this.inMax = el("input", "number", "max", div);
    this.smooth = el("input", "range", "smooth", div);
    this.output = el("input", "number", "out", div);
    this.id = data.index;
    this.input.disabled = true;
    this.output.disabled = true;

    for (var input of [this.input, this.inMin, this.inMax]) {
      input.addEventListener("change", e => {
        this.update();
      });
    }

    // keys
    var div = elDiv(this.view);
    this.keys = el("select", null, "key", div);
    this.keyThreshold = el("input", "number", "threshold", div, 1);
    this.keySend = el("input", "checkbox", "send key", div);
    this.keyPressed = false;

    // midi
    this.midiSend = el("input", "checkbox", "send midi", div);
    this.midiCC = el("input", "number", "cc", div);

    this.setPins();
    this.setKeys();

    this.pins.value = data.pin;
    this.inMin.value = data.inMin || 0;
    this.inMax.value = data.inMax || 1023;
    this.outMin = data.outMin || 0;
    this.outMax = data.outMax || 127;
    this.keys.value = data.key || "-";
    this.keyThreshold.value = data.keyThreshold || 1;
    this.keySend.checked = data.keySend || false;
    this.midiSend.checked = data.midiSend || false;
    this.midiCC.value = data.midiCC || 1;
  }

  update(val) {
    // var oldVal = Number(this.input.val);
    // if (oldVal !== val) {
    //   for (var i = 0; i < 10; i++) {
    //     var n = i / 10;
    //     oldVal = smoothstep(oldVal, val, i);
    //   }
    // }
    var a = Math.ceil(this.input.value);
    var b = lerp(a, val, this.smooth.value);
    b = Math.round(b);

    var mapped = b.map(
      this.inMin.value,
      this.inMax.value,
      this.outMin,
      this.outMax
    );

    mapped = Math.round(mapped);

    this.input.value = b;
    this.output.value = mapped;
  }

  setPins() {
    for (const pin of config.get("pins")) {
      let option = document.createElement("option");
      option.value = pin;
      option.textContent = pin;
      this.pins.appendChild(option);
    }
  }

  setKeys() {
    for (const key of config.get("keys")) {
      let option = document.createElement("option");
      option.value = key;
      option.textContent = key;
      this.keys.appendChild(option);
    }
  }

  calibrate() {
    var min = 9999;
    var max = 0;
    var val = Number(this.input.value);
    if (val > max) this.inMax.value = val;
    if (val < min) this.inMin.value = val;
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
