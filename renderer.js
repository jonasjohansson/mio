"use strict";
const { ipcRenderer } = require("electron");
const { BrowserWindow } = require("electron").remote;
const SerialPort = require("serialport");
const Readline = require("@serialport/parser-readline");
const midi = require("midi");
const robot = require("robotjs");
const config = require("./config");

const output = new midi.Output();
const parser = new Readline();
const keysAllowed = config.get("keys");

let port;
var keysPressed = [];
var keysIncoming = [];

let connectBtn, portSelect, baudSelect, devicesSelect;

var portOpen = false;
var midiOpen = false;

document.addEventListener("DOMContentLoaded", () => {
  connectBtn = document.querySelector("#connect");
  portSelect = document.querySelector("#ports");
  baudSelect = document.querySelector("#baudrates");
  devicesSelect = document.querySelector("#devices");

  /* Get baudrates */
  for (const baudrate of config.get("baudrates")) {
    var option = document.createElement("option");
    option.textContent = baudrate;
    baudSelect.appendChild(option);
  }

  scan();
  scanMidi();
});

function scan() {
  removeAllChildren(portSelect);
  SerialPort.list(function(err, ports) {
    ports = ports.filter(isArduino);
    connectBtn.disabled != ports.length;
    for (var port of ports) {
      var option = document.createElement("option");
      option.textContent = port.comName;
      if (port.comName.includes("usbmodem")) {
        option.selected = true;
      }
      portSelect.appendChild(option);
    }
  });
}

function scanMidi() {
  removeAllChildren(devicesSelect);
  var portCount = output.getPortCount();
  for (var i = 0; i < portCount; i++) {
    var option = document.createElement("option");
    option.textContent = output.getPortName(i);
    devicesSelect.appendChild(option);
  }
}

function connect(el) {
  if (portOpen === false) {
    port = new SerialPort(portSelect.value, {
      baudRate: Number(baudSelect.value),
      autoOpen: true,
      lock: false
    });
    port.on("open", function() {
      document.documentElement.classList.add("serial-connected");
      el.textContent = "disconnect";
      portOpen = true;
    });
    port.on("close", function() {
      document.documentElement.classList.remove("serial-connected");
      el.textContent = "connect";
      portOpen = false;
      // setTimeout(function() {
      //   scan();
      // }, 100);
    });
    port.on("error", err => {
      alert(err);
    });
    port.pipe(parser);
  } else {
    port.close();
  }
}

function connectMidi(el) {
  if (midiOpen === true) {
    output.closePort(devicesSelect.selectedIndex);
    midiOpen = false;
    el.textContent = "connect";
    document.documentElement.classList.remove("midi-connected");
  } else {
    midiOpen = true;
    output.openPort(devicesSelect.selectedIndex);
    document.documentElement.classList.add("midi-connected");
    el.textContent = "disconnect";
    output.sendMessage([176, 44, 127]);
    output.sendMessage([16, 1, 0]);
  }
}

parser.on("data", str => {
  str = str.trim();
  // return if string is too short or does not contain special symbol
  if (str.length < 2 || str[0] !== "$") return;

  var key = str.substr(1);
  var keyIndex = getIndex(key);

  if (keyIndex < 0) return;

  if (!keysPressed.includes(key)) {
    keysPressed.push(key);
    robot.keyToggle(key, "down");
    ipcRenderer.send("keyDown", key);
    output.sendMessage([16, 127, keyIndex]);
  }
  if (!keysIncoming.includes(key)) {
    keysIncoming.push(key);
  }
});

function getIndex(key) {
  return keysAllowed.indexOf(key);
}

function isArduino(port) {
  var p = port["vendorId"];
  return p !== undefined && p.includes("2341");
  // return pm !== undefined && port.comName.includes("usbmodem");
}

function removeAllChildren(node) {
  while (node.firstChild) {
    node.removeChild(node.firstChild);
  }
}

function log(msg, type = "") {
  var log = document.getElementById("log");
  log.innerHTML = msg;
  log.className = type;
}

function render() {
  requestAnimationFrame(render);

  for (var i = 0; i < keysPressed.length; i++) {
    var key = keysPressed[i];

    // if the key is not incoming any more, it must have been released
    if (!keysIncoming.includes(key)) {
      robot.keyToggle(key, "up");
      ipcRenderer.send("keyUp");
      var keyIndex = getIndex(key);
      output.sendMessage([16, 0, keyIndex]);
      var index = getIndex(key);
      keysPressed = keysPressed.filter(k => k !== key);
    }
  }
  keysIncoming = [];

  log(keysPressed);
}

requestAnimationFrame(render);
