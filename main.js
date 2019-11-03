"use strict";
const { app, BrowserWindow, ipcMain } = require("electron");
const electron = require("electron");
const path = require("path");
const url = require("url");
const config = require("./config");
const menu = require("./menu");

let win;
let isQuitting = false;

app.on("ready", () => {
  electron.Menu.setApplicationMenu(menu);
  createWindow();
});

app.on("window-all-closed", () => {
  app.quit();
});

app.on("activate", () => {
  win.show();
});

app.on("before-quit", () => {
  isQuitting = true;
  config.set("lastWindowState", win.getBounds());
});

function createWindow() {
  const lastWindowState = config.get("lastWindowState");

  win = new BrowserWindow({
    title: app.getName(),
    x: lastWindowState.x,
    y: lastWindowState.y,
    width: lastWindowState.width,
    height: lastWindowState.height,
    minWidth: 400,
    minHeight: 200,
    // titleBarStyle: 'hiddenInset',
    // frame: false,
    alwaysOnTop: config.get("alwaysOnTop"),
    webPreferences: {
      nodeIntegration: true
    }
  });

  win.loadURL(`file://${__dirname}/index.html`);

  win.on("close", event => {
    if (!isQuitting) {
      event.preventDefault();
      app.hide();
    }
  });
}

ipcMain.on("quit", () => {
  app.quit();
});

ipcMain.on("debug", () => {
  win.webContents.openDevTools({ mode: "detach" });
});
