class SimpleController(
    private val serviceC: ServiceC,
) {

    companion object {
        private val ANOTHER_FLAG =
            createEmptyToggles(
                id = 1,
                userId = 2
            )
    }

    private val someProp: String
    private val anotherProp: String

    @MockkBean
    private lateinit var serviceF: TestClass

    inner class SomeInnerClass {

        fun helloWorld() {
            print("hello")
        }
    }

    fun main() {
        println("hello world")
        serviceC.monitor(someProp, "test1",
            "test2", anotherProp, "test3")
    }

    fun main2() {
        print("hello")
        serviceF.monitor()
    }

    fun someExpectation() {
        val result = service.doSomething()
        result shouldBe ANOTHER_FLAG
    }
}
