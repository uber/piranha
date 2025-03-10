class SimpleController(
    private val sericeA: ServiceA,
    private val featureService: FeatureService,
    private val serviceB: ServiceB,
    private val serviceC: ServiceC,
    private val serviceD: ServiceD,
) {

    companion object {
        val TEST = 1

        private val EMPTY_TOGGLES =
            createEmptyToggles(
                id = 1,
                userId = 2
            )

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

    @MockkBean(relaxed = True)
    private lateinit var serviceE: TestClass

    @MockkBean(relaxed = True)
    private lateinit var serviceP: TestClass

    @MockkBean
    private lateinit var serviceF: TestClass

    private val someService = mock<SomeService> {
        on { something(someUserId) } doReturn someTestEligibility
    }

    private val serviceK: ServiceK = mockk(relaxed = true)

    private val serviceM: ServiceM = mockk(relaxed = true)

    private val compositeMonitor =
        CompositeMonitor(
            serviceK,
            serviceM
        )

    inner class SomeInnerClass {

        fun helloWorld() {
            print("hello")
        }
    }

    fun main() {
        if (featureService.isEnabled(STALE_FLAG)) {
            println("hello world")
        } else {
            serviceE.monitor()
            println("ok")
        }
        serviceC.monitor(someProp, "test1",
            "test2",
            anotherProp, "test3")
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
