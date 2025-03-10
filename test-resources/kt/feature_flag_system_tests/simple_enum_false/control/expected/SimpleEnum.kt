enum class FeatureFlags(val someTestValue: Boolean) {
    FEATURE_B(true)
    ;
}

fun testToggleMethod(criteria: Boolean) =
    emptyList()

fun featureEnabled_return_expression_and_something_else() {
    print("ok")
    return listOf(1, 2, 3)
}

fun featureEnabled_return_expression() {
    print("ok")
    return listOf(1, 2, 3)
}

fun featureEnabled_return_expression() {
    print("ok")
    return listOf(1, 2, 3)
}

fun Map<String, String>.isSomeExpressionEnabled(key: String, defaultValue: Boolean): Boolean {
    return false
}

fun main() {
    println("Hi world!")
    println("Hi world!")
    println("Hi world!")
    println("Hi world!")
}
