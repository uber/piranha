class CleanedUpController < ApplicationController
    before_action :keep_if_action, only: [:create, :update]
    before_action :set_variable
    before_action :authenticate_user!
    before_action :authenticate_user_with_params!
  
    before_action :do_not_change, only: [:create, :update], if: -> { check_something? }
  end