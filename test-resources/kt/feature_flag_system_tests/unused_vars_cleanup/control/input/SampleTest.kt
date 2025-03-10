class SampleTest {
    private val featureService
    @Autowired
    private lateinit var fermiumStub: FermiumStub
    @Autowired
    private lateinit var anotherStub: FermiumStub

    @Test
    fun testSomething() {
        val i = 0
        assertTrue(featureService.isEnabled(STALE_FLAG))
    }

    @Test
    fun testSomething2() {
        val i = 0
        fermiumStub.setEnabled(STALE_FLAG, false)
    }

    @Test
    fun testSomething2() {
        val i = 0
        anotherStub.setEnabled(STALE_FLAG, true)
    }
}
