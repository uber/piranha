interface SomeParameter {

    @BoolParam(key="STALE_FLAG", namespace = "some_long_name")
    BoolParameter isStaleFeature();

    @BoolParam(key="other_flag", namespace = "some_long_name")
    BoolParameter isOtherFlag();

}