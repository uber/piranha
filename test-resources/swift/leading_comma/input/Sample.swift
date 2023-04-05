enum TestEnum {
   case one
   case two
   case three
   case four

   var v1: String {
      switch self {
      case .two, .one, .four:
         return "Hello"
      case .three:
         return "Hi"
      }
   }

   var v2: String {
      switch self {
         case .two, .one:
            return "world"
         case .three, .four:
            return "World!"
         }
   }
}
