

                            </button>
                        </div>
                    }.into_any();
                }


                view! {
                    <form class="auth-form" on:submit=handle_submit>


                        <div class="form-group">
                            <label class="form-label">"Email"</label>
                            <div class="input-with-icon">
                                <IconMail class="input-icon icon-svg" />
                                <input
                                    class="form-input form-input--icon"
                                    type="email"
                                    placeholder="you@example.com"
                                    prop:value=move || email.get()
                                    on:input=move |ev| set_email.set(event_target_value(&ev))
                                    required
                                />
                            </div>
                        </div>


                        <div class="form-group">
                            <label class="form-label">"Password"</label>
                            <div class="input-with-icon">
                                <IconLock class="input-icon icon-svg" />
                                <input
                                    class="form-input form-input--icon"
                                    type="password"
                                    placeholder="••••••••"
                                    prop:value=move || password.get()
                                    on:input=move |ev| set_password.set(event_target_value(&ev))
                                    required
                                />
                            </div>
                            <div class="forgot-link-row">
                                <button
                                    type="button"
                                    class="link-btn link-btn--muted"
                                    on:click=move |_| on_forgot()
                                >
                                    "Forgot password?"
                                </button>
                            </div>
                        </div>


                        {move || if !error.get().is_empty() {
                            view! {
                                <div class="status-msg status-msg--error">{error.get()}</div>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}


                        {move || if login_state.get() == LoginState::Unverified {
                            view! {
                                <div class="verify-banner">
                                    <div class="verify-banner-text">
                                        <strong>"Email not verified."</strong>
                                        " Check your inbox for the confirmation link."
                                    </div>
                                    <button
                                        type="button"
                                        class="btn btn-ghost btn-sm verify-resend-btn"
                                        disabled=move || loading.get()
                                        on:click=handle_resend
                                    >
                                        {move || if loading.get() {
                                            view! {
                                                <IconLoader class="icon-svg spin" />
                                                <span>"Sending..."</span>
                                            }.into_any()
                                        } else {
                                            view! { <span>"Resend verification email"</span> }.into_any()
                                        }}
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}


                        <button
                            type="submit"
                            class="btn btn-primary btn-full"
                            disabled=move || loading.get()
                        >
                            {move || if loading.get() && login_state.get() != LoginState::Unverified {
                                view! {
                                    <IconLoader class="icon-svg spin" />
                                    <span>"Signing in..."</span>
                                }.into_any()
                            } else {
                                view! { <span>"Sign In"</span> }.into_any()
                            }}
                        </button>


                    </form>


                    <div class="auth-switch">
                        "Don't have an account? "
                        <button class="link-btn" on:click=move |_| on_switch()>
                            "Create one"
                        </button>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
