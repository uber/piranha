object AddTwoNumbers {
  def main(args: Array[String]): Unit = {
    // Read input from the user
    print("Enter first number: ")
    val num1 = scala.io.StdIn.readDouble()

    print("Enter second number: ")
    val num2 = scala.io.StdIn.readDouble()

    // Calculate the sum
    val sum = num1 + num2

    // Display the result
    println(s"The sum of $num1 and $num2 is: $sum")
  }
}
