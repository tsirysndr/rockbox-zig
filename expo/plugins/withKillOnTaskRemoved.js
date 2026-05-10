const { withMainActivity } = require("@expo/config-plugins");

// Kill the process when the user swipes the app away from recents.
//
// Android destroys the root Activity on swipe-away. isTaskRoot is true when
// this is the only activity left in the task (always true with singleTask).
// isChangingConfigurations guards against killing during screen rotation.
// Process.killProcess ensures the foreground service, Tokio runtime, and
// embedded firmware threads all die immediately — so the next reopen is a
// completely fresh start with clean gRPC connections.
const KILL_ON_TASK_REMOVED = `
  override fun onDestroy() {
    super.onDestroy()
    if (isTaskRoot && !isChangingConfigurations) {
      android.os.Process.killProcess(android.os.Process.myPid())
    }
  }
`;

const withKillOnTaskRemoved = (config) => {
  return withMainActivity(config, (config) => {
    let contents = config.modResults.contents;

    if (contents.includes("killProcess")) {
      // Already applied (e.g. manual edit before prebuild).
      return config;
    }

    // Insert the override right before the first `override fun` in the class
    // body so it lands at a safe, stable anchor point regardless of what
    // expo-router or other plugins add to MainActivity.
    contents = contents.replace(
      /(\s{2}override fun onCreate\()/,
      `${KILL_ON_TASK_REMOVED}\n  $1`,
    );

    config.modResults.contents = contents;
    return config;
  });
};

module.exports = withKillOnTaskRemoved;
