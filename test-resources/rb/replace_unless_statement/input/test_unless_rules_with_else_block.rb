def test_unless_statement_true_with_else
    unless true
        do_something
    else
        do_something_else
    end
end

def test_unless_statement_true_parenthesized_with_else
    unless(true)
        do_something
    else
        do_something_else
    end
end

def test_unless_statement_false_with_else
  unless false
      do_something
  else
      do_something_else
  end
end

def test_unless_statement_false_parenthesized_with_else
  unless(false)
      do_something
  else
      do_something_else
  end
end