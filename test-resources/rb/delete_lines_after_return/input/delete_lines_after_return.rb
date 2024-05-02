def remove_multiple_lines_after_return
  before_return
  return super
  after_return
  another_method_call
end
