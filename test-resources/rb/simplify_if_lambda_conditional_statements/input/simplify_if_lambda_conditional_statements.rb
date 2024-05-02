class CleanedUpController < ApplicationController
    before_action :keep_if_action, only: [:create, :update], if: -> { true }
    before_action :set_variable, if: ->(c) { true }
    before_action :authenticate_user!, if: lambda { true }
    before_action :authenticate_user_with_params!, if: lambda { |c| true }

    before_action :keep_if_action, only: [:create, :update], if: -> { false }
    before_action :set_variable, if: ->(c) { false }
    before_action :authenticate_user!, if: lambda { false }
    before_action :authenticate_user_with_params!, if: lambda { |c| false }

    before_action :do_not_change, only: [:create, :update], if: -> { check_something? }
end