def test_if_ternary
  true ? do_something : do_something_else
end

def test_if_ternary_negeated
  false ? do_something : do_something_else
end

def test_if_ternary_parenthesized
  (true) ? do_something : do_something_else
end

def test_if_ternary_negeated_parenthesized
  (false) ? do_something : do_something_else
end