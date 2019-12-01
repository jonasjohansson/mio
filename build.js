"use strict";

const builder = require("electron-builder");
const Platform = builder.Platform;

// https://github.com/electron-userland/electron-builder#programmatic-usage
// https://www.electron.build/configuration/dmg
// Promise is returned
builder
  .build({
    targets: Platform.MAC.createTarget(),
    // https://github.com/electron-userland/electron-builder/issues/1313
    // targets: Platform.WINDOWS.createTarget('portable'),
    config: {
      appId: "se.jonasjohansson.mio",
      mac: {
        category: "public.app-category.social-networking",
        target: ["dir", "dmg", "zip"],
        hardenedRuntime: true,
        entitlements: "./build/entitlements.mac.inherit.plist",
        electronUpdaterCompatibility: ">=2.16.0"
        // extendInfo: {
        //   LSUIElement: 1
        // }
      },
      dmg: {
        title: "${name}",
        backgroundColor: "#999",
        // icon: 'build/icon-negative.png',
        iconSize: 100
      }
    }
  })
  .then(() => {
    // handle result
  })
  .catch(error => {
    // handle error
  });

// build options, see https://goo.gl/QQXmcV
// https://www.electron.build/configuration/mac
