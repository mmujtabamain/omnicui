#!/bin/sh
# Builds the android target once, so that `omnicui check android` starts a plain JVM and never
# Gradle: compiles the adapter (and the app's ui library, an included build), writes the test
# runtime classpath into build/adapter.args, then starts the adapter once to fetch Robolectric's
# Android 16 jar (the first time only, about 200 MB into ~/.m2) and to time its start-up.
set -eu
cd "$(dirname "$0")"
: "${ANDROID_HOME:=$HOME/.local/android-sdk}"
export ANDROID_HOME
start=$(date +%s)
./gradlew --console=plain -q adapterClasspath
built=$(date +%s)
echo "android: built in $((built - start)) s; classpath in build/adapter.args"
t0=$(date +%s%N)
reply=$(printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"advertise","params":{"protocol":"1"}}' \
  | java -Xmx2g -XX:+UseParallelGC @build/adapter.args omnicui.android.Main 2>/dev/null | head -n 1)
t1=$(date +%s%N)
case "$reply" in
  *'"protocol":"1"'*) echo "android: the adapter answered advertise in $(( (t1 - t0) / 1000000 )) ms (start-up, JVM to first reply)" ;;
  *) echo "android: the adapter didn't answer advertise: $reply" >&2; exit 1 ;;
esac
