enum class FeatureFlags(val someTestValue: Boolean) {
    FEATURE_B(true)
    ;
}

fun testToggleMethod(criteria: Boolean) =
    if (critera) {
        println("ok")
    } else {
        println("not ok")
    }

fun featureEnabled_return_expression_2() {
    print("not ok")
    return listOf(1, 2, 3)
}

fun featureEnabled_return_expression() {
    val i = 0
    print("ok")
    print("ok")
    return listOf(1, 2, 3)
}

fun Map<String, String>.isSomeExpressionEnabled(key: String, defaultValue: Boolean): Boolean {
    return false
}

fun main() {
    println("Hello world!")
    println("Hi world!")
    println("Hi world!")
    println("Hi world!")
}
