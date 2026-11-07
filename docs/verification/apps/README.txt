The real apps, run through the run scripts on Linux under Xvfb (egui on lavapipe), then driven
with real X input: "+" pressed and the switch clicked.
  ./run.sh app desktop          → desktop-egui-after-clicks.png (cargo's build progress shown first)
  ./run.sh desktop-javafx app   → desktop-javafx-after-clicks.png (either word order works)
  web-after-clicks.png: the web page in Chromium, with "+" pressed twice, the switch on and the
  dark colour scheme ticked.
