'use strict'

const { app, BrowserWindow, ipcMain } = require('electron')
const electron = require('electron')
const menu = require('./menu')
const config = require('./config')

let win

const width = 200
const height = 500

app.disableHardwareAcceleration()

app.allowRendererProcessReuse = false

app.on('ready', () => {
    electron.Menu.setApplicationMenu(menu)
    createWindow()
})

app.on('window-all-closed', () => {
    app.quit()
})

app.on('activate', () => {
    win.show()
})

app.on('before-quit', () => {
    config.set('lastWindowState', win.getBounds())
})

function createWindow() {
    const lastWindowState = config.get('lastWindowState')
    win = new BrowserWindow({
        title: app.name,
        x: lastWindowState.x,
        y: lastWindowState.y,
        width: width,
        minWidth: width,
        maxWidth: width,
        height: height,
        minHeight: height,
        titleBarStyle: 'customButtonsOnHover',
        frame: false,
        resizeable: false,
        transparent: true,
        webPreferences: {
            nodeIntegration: true
        }
    })

    app.dock.hide()
    win.setAlwaysOnTop(true, 'floating')
    win.setVisibleOnAllWorkspaces(true)
    win.fullScreenable = false
    app.dock.show()

    win.loadURL(`file://${__dirname}/index.html`)
}

ipcMain.on('quit', () => {
    app.quit()
})
