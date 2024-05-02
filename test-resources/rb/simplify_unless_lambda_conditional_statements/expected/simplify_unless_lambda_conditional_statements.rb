class CleanedUpController < ApplicationController
  before_action :unless_staby_syntax_keep_unless_action, only: [:create, :update]
  before_action :unless_staby_syntax_set_variable
  before_action :unless_staby_syntax_authenticate_user!
  before_action :unless_staby_syntax_authenticate_user_with_params!

  before_action :do_not_change, only: [:create, :update], unless: -> { check_something? }
end