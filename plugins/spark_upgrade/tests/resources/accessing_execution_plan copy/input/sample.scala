package org.piranha

object AccessingExecutionPlan {
  def checkAnswer(actual: => DataFrame, expectedAnswer: Seq[Row]): Unit = {
    QueryTest.checkAnswer(actual, expectedAnswer) match {
      case Some(errorMessage) => fail(errorMessage)
      case None =>
    }
  }

  def checkAnswer1(actual: => DataFrame, expectedAnswer: Seq[Row]): Unit = {
    QueryTest.checkAnswer(actual, expectedAnswer) match {
      case Some(errorMessage) => fail(errorMessage)
      case None => // ignore
    }
  }
}
