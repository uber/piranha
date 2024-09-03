def test_unless_statement_true_without_else
    unless true
        do_something
    end
end

def test_unless_statement_true_parenthesized_without_else
    unless(true)
        do_something
    end
end

def test_unless_statement_false_without_else
  unless false
      do_something
  end
end

def test_unless_statement_false_parenthesized_without_else
  unless(false)
      do_something_else
  end
end
