enum TestEnum {
   case one
   case two
   case three

   var v1: String {
      switch self {
      case .two, .one:
         return "Hello"
      case .three:
         return "Hi"
      }
   }
}
