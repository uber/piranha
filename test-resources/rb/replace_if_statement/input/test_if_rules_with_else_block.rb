def test_if_statement_true_with_else
    if true
        do_something
    else
        do_something_else
    end
end

def test_if_statement_true_parenthesized_with_else
    if(true)
        do_something
    else
        do_something_else
    end
end

def test_if_statement_false_with_else
  if false
      do_something
  else
      do_something_else
  end
end

def test_if_statement_false_parenthesized_with_else
  if(false)
      do_something
  else
      do_something_else
  end
end

def test_if_statement_with_elsif
  if check_if_true?
    do_something
  elsif true
    do_something_else
  else
    do_something_else_again
  end
end
