def test_cleanup_when_assigned_to_local_variable
  
  do_flag_enabled_stuff
end

def test_cleanup_when_assigned_to_local_variable_and_redeclared
  is_flag_enabled = true
  is_flag_enabled = check_another_condition?
  if(is_flag_enabled)
    do_flag_enabled_stuff
  else
    do_non_flag_stuff
  end
end

def test_cleanup_when_assigned_to_instance_variable

    do_flag_enabled_stuff
end

def test_cleanup_when_assigned_to_instance_variable_and_redeclared
  @is_flag_enabled = true
  @is_flag_enabled = check_another_condition?
  if(@is_flag_enabled)
    do_flag_enabled_stuff
  else
    do_non_flag_stuff
  end
end
