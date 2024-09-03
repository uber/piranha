def test_if_statement_true_without_else
    if true
        do_something
    end
end

def test_if_statement_true_parenthesized_without_else
    if(true)
        do_something
    end
end

def test_if_statement_false_without_else
  if false
      do_something
  end
end

def test_if_statement_false_parenthesized_without_else
  if(false)
      do_something_else
  end
end
