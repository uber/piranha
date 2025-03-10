import com.package.FeatureFlags.STALE_FLAG

fun test() {
    if (featureService.isEnabled(STALE_FLAG)) {
        print("hello world")
    }
}
