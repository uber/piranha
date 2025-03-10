enum class FeatureFlags(val someTestValue: Boolean) {
    STALE_FLAG(true),
    FEATURE_B(true)
    ;
}

fun isFeatureEnabled_simple_enum() =
    featureService.isEnabled(STALE_FLAG.name)

fun isNotFeatureEnabled_simple_enum() =
    featureService.isEnabled(FeatureFlags.STALE_FLAG)

fun isNotFeatureEnabled_full_qualified_simple_enum() =
    featureService.isEnabled(PackageA.FeatureFlags.STALE_FLAG.key)

fun isNotFeatureEnabled_super_full_qualified_simple_enum() =
    featureService.isEnabled(PackageB.FeatureFlags.STALE_FLAG.key)

fun testToggleMethod(criteria: Boolean) =
    if (featureService.isFeatureEnabled(STALE_FLAG)) {
        if (critera) {
            println("ok")
        } else {
            println("not ok")
        }
    } else {
        emptyList()
    }



fun featureEnabled_return_expression_2() {
    return if (!featureService.isEnabled(STALE_FLAG)) {
        val i = 0
        print("ok")
        print("ok")
        listOf(1, 2, 3)
    } else {
        print("not ok")
        listOf(1, 2, 3)
    }
}

fun featureEnabled_return_expression() {
    return if (featureService.isEnabled(STALE_FLAG)) {
        val i = 0
        print("ok")
        print("ok")
        listOf(1, 2, 3)
    } else {
        print("not ok")
        listOf(1, 2, 3)
    }
}

fun Map<String, String>.isSomeExpressionEnabled(key: String, defaultValue: Boolean): Boolean {
    return key == PackageA.FeatureFlags.STALE_FLAG.key && defaultValue
}

fun main() {
    if (isFeatureEnabled_simple_enum()) {
        println("Hello world!")
    } else {
        println("Hi world!")
    }

    if (isNotFeatureEnabled_simple_enum()) {
        println("Hello world!")
    } else {
        println("Hi world!")
    }

    if (isNotFeatureEnabled_full_qualified_simple_enum()) {
        println("Hello world!")
    } else {
        println("Hi world!")
    }

    if (isNotFeatureEnabled_super_full_qualified_simple_enum()) {
        println("Hello world!")
    } else {
        println("Hi world!")
    }

    val test = PackageA.FeatureFlags.STALE_FLAG.key
}
