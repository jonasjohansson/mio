"use strict";
const { app, BrowserWindow, ipcMain } = require("electron");
const electron = require("electron");
const path = require("path");
const url = require("url");
const config = require("./config");
// const menu = require("./menu");
// const tray = require("./tray");

const trayWindow = require("electron-tray-window");

let win;
let isQuitting = false;

app.on("ready", () => {
  // electron.Menu.setApplicationMenu(menu);
  trayWindow.setOptions({
    trayIconPath: "assets/icons/trayIcon.png",
    windowUrl: `file://${__dirname}/index.html`
  });
  createWindow();
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
    show: false
  });

  win.loadURL(`file://${__dirname}/index.html`);

  win.on("close", event => {
    config.set("lastWindowState", win.getBounds());
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
