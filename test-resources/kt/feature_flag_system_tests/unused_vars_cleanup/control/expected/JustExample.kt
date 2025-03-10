package some.example

import org.springframework.http.HttpStatus
import java.util.UUID

internal class SomeTest(
    @Value("\${some.prop1}")
    private val prop1: String,
) : IntegrationTest() {

    companion object {
    }

    @BeforeAll
    fun setUp() {
        println("setup")
    }

    @ParameterizedTest(name = "{index} ==> some test name")
    @EnumSource(SomeEnum::class)
    fun `should work fine`(operation: SomeOperation) {
        checkSomething()
        verifySomething()
    }
}
