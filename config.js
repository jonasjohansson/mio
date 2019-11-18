"use strict";
const Store = require("electron-store");

module.exports = new Store({
  defaults: {
    lastWindowState: {
      x: 0,
      y: 0,
      width: 800,
      height: 600
    },
    advancedMode: false,
    alwaysOnTop: false,
    ios: [
      {
        pin: "d3",
        inMin: 0,
        inMax: 1,
        outMin: 0,
        outMax: 127,
        threshold: 64
      },
      {
        pin: "d4",
        inMin: 0,
        inMax: 1,
        outMin: 0,
        outMax: 127,
        threshold: 64
      }
    ],
    pins: [
      "-",
      "a0",
      "a1",
      "a2",
      "a3",
      "a4",
      "a5",
      "d0",
      "d1",
      "d2",
      "d3",
      "d4",
      "d5",
      "d6",
      "d7",
      "d8",
      "d9",
      "d10",
      "d11",
      "d12",
      "d13"
    ],
    baudrates: [9600, 115200],
    keys: [
      "a",
      "b",
      "c",
      "d",
      "e",
      "f",
      "g",
      "h",
      "i",
      "j",
      "k",
      "l",
      "m",
      "n",
      "o",
      "p",
      "q",
      "r",
      "s",
      "t",
      "u",
      "v",
      "w",
      "x",
      "y",
      "z",
      "alt",
      "ctrl",
      "space",
      "shift",
      "enter",
      "space",
      "up",
      "down",
      "left",
      "right"
    ]
  }
});
