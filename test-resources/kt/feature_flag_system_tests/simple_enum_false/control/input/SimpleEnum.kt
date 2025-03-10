enum class FeatureFlags(val someTestValue: Boolean) {
    STALE_FLAG(true),
    FEATURE_B(true)
    ;
}

fun isFeatureEnabled_simple_enum() =
    featureService.isEnabled(STALE_FLAG.name)

fun isFeatureEnabled_simple_enum_2() =
    featureService.isEnabled(FeatureFlags.STALE_FLAG)

fun isFeatureEnabled_full_qualified_simple_enum() =
    featureService.isEnabled(PackageA.FeatureFlags.STALE_FLAG.key)

fun isFeatureEnabled_super_full_qualified_simple_enum() =
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


fun featureEnabled_return_expression_and_something_else() {
    return if (featureService.isEnabled(STALE_FLAG) && checkSomethingElse()) {
        val fuck = 0
        print("not ok")
        print("not ok")
        listOf(1, 2, 3)
    } else {
        print("ok")
        listOf(1, 2, 3)
    }
}

fun featureEnabled_return_expression() {
    return if (featureService.isEnabled(STALE_FLAG)) {
        val i = 0
        print("not ok")
        print("not ok")
        listOf(1, 2, 3)
    } else {
        print("ok")
        listOf(1, 2, 3)
    }
}

fun featureEnabled_return_expression() {
    return if (!featureService.isEnabled(STALE_FLAG)) {
        print("ok")
        listOf(1, 2, 3)
    } else {
        val i = 0
        print("not ok")
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

    if (isFeatureEnabled_simple_enum_2()) {
        println("Hello world!")
    } else {
        println("Hi world!")
    }

    if (isFeatureEnabled_full_qualified_simple_enum()) {
        println("Hello world!")
    } else {
        println("Hi world!")
    }

    if (isFeatureEnabled_super_full_qualified_simple_enum()) {
        println("Hello world!")
    } else {
        println("Hi world!")
    }

    val test = PackageA.FeatureFlags.STALE_FLAG.key
}
