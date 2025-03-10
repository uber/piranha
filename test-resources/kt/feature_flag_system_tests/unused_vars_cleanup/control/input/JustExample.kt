package some.example

import org.springframework.http.HttpStatus
import java.util.UUID

internal class SomeTest(
    @Value("\${some.prop1}")
    private val prop1: String,
) : IntegrationTest() {

    companion object {

        private const val someConst = "val 1"
        private const val someCOnst2 = "val 2"
    }

    @Autowired
    private lateinit var service1: SomeService

    @Autowired
    private lateinit var objectMapper: ObjectMapper

    @Autowired
    private lateinit var repository1: SomeRepository

    @Autowired
    private lateinit var featureFlagStub: FeatureFlagStub

    @BeforeAll
    fun setUp() {
        println("setup")
        featureFlagStub.setFeature(Feature(FeatureFlagsAnother.ANOTHER_REPRESENTATION_OF_STALE_FLAG.featureFlagName), true)
    }

    @ParameterizedTest(name = "{index} ==> some test name")
    @EnumSource(SomeEnum::class)
    fun `should work fine`(operation: SomeOperation) {
        checkSomething()
        verifySomething()
    }
}
