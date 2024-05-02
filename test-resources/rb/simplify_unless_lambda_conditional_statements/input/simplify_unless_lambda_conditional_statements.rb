class CleanedUpController < ApplicationController
    before_action :unless_lambda_syntax_keep_unless_action, only: [:create, :update], unless: -> { true }
    before_action :unless_lambda_syntax_set_variable, unless: ->(c) { true }
    before_action :unless_lambda_syntax_authenticate_user!, unless: lambda { true }
    before_action :unless_lambda_syntax_authenticate_user_with_params!, unless: lambda { |c| true }

    before_action :unless_staby_syntax_keep_unless_action, only: [:create, :update], unless: -> { false }
    before_action :unless_staby_syntax_set_variable, unless: ->(c) { false }
    before_action :unless_staby_syntax_authenticate_user!, unless: lambda { false }
    before_action :unless_staby_syntax_authenticate_user_with_params!, unless: lambda { |c| false }

    before_action :do_not_change, only: [:create, :update], unless: -> { check_something? }
end