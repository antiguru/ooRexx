::routine go public
dir = filespec('location', .context~package~name)
r = .routine~new(.context~package~name, ("return hello()", "::requires 'y_st.cls'"), .context~package)
return r~call
