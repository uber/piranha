func logTestMessage() {

	logger.With(zap.Any("field", complex128(0))).Info("test")
	c1 = float64(0)
	c = complex128(0)
	logger.With(zap.Any("field", &c1)).Info("test")
}
