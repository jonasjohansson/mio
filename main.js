"use strict";

const { app, BrowserWindow, ipcMain } = require("electron");
const electron = require("electron");
const menu = require("./menu");
const config = require("./config");

let win;

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
    minWidth: 200,
    minHeight: 300,
    webPreferences: {
      nodeIntegration: true
    }
  });

  win.loadURL(`file://${__dirname}/index.html`);
}

ipcMain.on("quit", () => {
  app.quit();
});
