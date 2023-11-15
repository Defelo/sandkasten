{time}:
time.overrideAttrs {
  patches = [./precision.patch];
}
