class SimpleController(
    @SuppressWarnings("UnusedPrivateProperty") private val sericeA: ServiceA,
    @SuppressWarnings("UnusedPrivateProperty") private val featureService: FeatureService,
    @SuppressWarnings("UnusedPrivateProperty") private val serviceB: ServiceB,
    private val serviceC: ServiceC,
    @SuppressWarnings("UnusedPrivateProperty") private val serviceD: ServiceD,
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

    override val someOverridenProp get() = true

    private val someTestEligibility = createEligibility()

    private val useCase = SomeUseCase(
        service1,
        service2,
        service3
    )

    private val stringConstant

    @MockkBean
    private lateinit var serviceF: TestClass

    private val someService = mock<SomeService> {
        on { something(someUserId) } doReturn someTestEligibility
    }

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
        someService()
        useCase.invoke(request)
        print("$stringConstant")
    }

    fun someExpectation() {
        val result = service.doSomething()
        result shouldBe ANOTHER_FLAG
    }
}

class Nothing {
}
