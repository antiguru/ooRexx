
/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 1995, 2004 IBM Corporation. All rights reserved.             */
/* Copyright (c) 2005-2017 Rexx Language Association. All rights reserved.    */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* http://www.oorexx.org/license.html                                         */
/*                                                                            */
/* Redistribution and use in source and binary forms, with or                 */
/* without modification, are permitted provided that the following            */
/* conditions are met:                                                        */
/*                                                                            */
/* Redistributions of source code must retain the above copyright             */
/* notice, this list of conditions and the following disclaimer.              */
/* Redistributions in binary form must reproduce the above copyright          */
/* notice, this list of conditions and the following disclaimer in            */
/* the documentation and/or other materials provided with the distribution.   */
/*                                                                            */
/* Neither the name of Rexx Language Association nor the names                */
/* of its contributors may be used to endorse or promote products             */
/* derived from this software without specific prior written permission.      */
/*                                                                            */
/* THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS        */
/* "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT          */
/* LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS          */
/* FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT   */
/* OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,      */
/* SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED   */
/* TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA,        */
/* OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY     */
/* OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING    */
/* NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS         */
/* SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.               */
/*                                                                            */
/*----------------------------------------------------------------------------*/
/******************************************************************************/
/* REXX Kernel                                                  okerrtxt.h    */
/*                                                                            */
/* Create table of mappings between Rexx errors and error messages            */
/*                                                                            */
/******************************************************************************/

#ifdef ERROR_TABLE

#define Table_end   0                  /* constant to mark end of table     */
#define Table_end_msg 0

ERROR_MESSAGE Message_table[] = {      /* table of major and minor errors   */


/* -------------------------------------------------------------------------- */
/* --            file is generated by build process                        -- */
/* --            ==================================================        -- */
/* --            DO NOT CHANGE THIS FILE, ALL CHANGES WILL BE LOST!        -- */
/* --            ==================================================        -- */
/* -------------------------------------------------------------------------- */

  MAJOR(Error_Program_unreadable)
      MINOR(Error_Program_unreadable_name)
      MINOR(Error_Program_unreadable_user_defined)
      MINOR(Error_Program_unreadable_notfound)
      MINOR(Error_Program_unreadable_output_error)
      MINOR(Error_Program_unreadable_version)
      MINOR(Error_Program_not_tokenized)
  MAJOR(Error_Program_interrupted)
      MINOR(Error_Program_interrupted_condition)
      MINOR(Error_Program_interrupted_user_defined)
  MAJOR(Error_System_resources)
      MINOR(Error_System_resources_user_defined)
  MAJOR(Error_Unmatched_quote)
      MINOR(Error_Unmatched_quote_comment)
      MINOR(Error_Unmatched_quote_single)
      MINOR(Error_Unmatched_quote_double)
      MINOR(Error_Unmatched_quote_user_defined)
  MAJOR(Error_When_expected)
      MINOR(Error_When_expected_when)
      MINOR(Error_When_expected_whenotherwise)
      MINOR(Error_When_expected_nootherwise)
  MAJOR(Error_Unexpected_then)
      MINOR(Error_Unexpected_then_then)
      MINOR(Error_Unexpected_then_else)
  MAJOR(Error_Unexpected_when)
      MINOR(Error_Unexpected_when_when)
      MINOR(Error_Unexpected_when_otherwise)
  MAJOR(Error_Unexpected_end)
      MINOR(Error_Unexpected_end_nodo)
      MINOR(Error_Unexpected_end_control)
      MINOR(Error_Unexpected_end_nocontrol)
      MINOR(Error_Unexpected_end_then)
      MINOR(Error_Unexpected_end_else)
      MINOR(Error_Unexpected_end_select)
      MINOR(Error_Unexpected_end_select_nolabel)
  MAJOR(Error_Control_stack)
      MINOR(Error_Control_stack_full)
      MINOR(Error_Control_stack_user_defined)
  MAJOR(Error_Invalid_character)
      MINOR(Error_Invalid_character_char)
      MINOR(Error_Invalid_character_user_defined)
  MAJOR(Error_Incomplete_do)
      MINOR(Error_Incomplete_do_do)
      MINOR(Error_Incomplete_do_select)
      MINOR(Error_Incomplete_do_then)
      MINOR(Error_Incomplete_do_else)
      MINOR(Error_Incomplete_do_otherwise)
      MINOR(Error_Incomplete_do_loop)
  MAJOR(Error_Invalid_hex)
      MINOR(Error_Invalid_hex_hexblank)
      MINOR(Error_Invalid_hex_binblank)
      MINOR(Error_Invalid_hex_invhex)
      MINOR(Error_Invalid_hex_invbin)
  MAJOR(Error_Label_not_found)
      MINOR(Error_Label_not_found_name)
  MAJOR(Error_Unexpected_procedure)
      MINOR(Error_Unexpected_procedure_call)
      MINOR(Error_Unexpected_procedure_interpret)
  MAJOR(Error_Then_expected)
      MINOR(Error_Then_expected_if)
      MINOR(Error_Then_expected_when)
  MAJOR(Error_Symbol_or_string)
      MINOR(Error_Symbol_or_string_address)
      MINOR(Error_Symbol_or_string_call)
      MINOR(Error_Symbol_or_string_name)
      MINOR(Error_Symbol_or_string_signal)
      MINOR(Error_Symbol_or_string_stream)
      MINOR(Error_Symbol_or_string_trace)
      MINOR(Error_Symbol_or_string_parse)
      MINOR(Error_Symbol_or_string_user_defined)
      MINOR(Error_Symbol_or_string_class)
      MINOR(Error_Symbol_or_string_method)
      MINOR(Error_Symbol_or_string_routine)
      MINOR(Error_Symbol_or_string_requires)
      MINOR(Error_Symbol_or_string_external)
      MINOR(Error_Symbol_or_string_metaclass)
      MINOR(Error_Symbol_or_string_subclass)
      MINOR(Error_Symbol_or_string_inherit)
      MINOR(Error_Symbol_or_string_tilde)
      MINOR(Error_Symbol_or_string_colon)
      MINOR(Error_Symbol_or_string_mixinclass)
      MINOR(Error_Symbol_or_string_attribute)
      MINOR(Error_Symbol_or_string_constant)
      MINOR(Error_Symbol_or_string_constant_value)
      MINOR(Error_Symbol_or_string_digits_value)
      MINOR(Error_Symbol_or_string_fuzz_value)
      MINOR(Error_Symbol_or_string_trace_value)
      MINOR(Error_Symbol_or_string_resource)
      MINOR(Error_Symbol_or_string_resource_end)
      MINOR(Error_Symbol_or_string_keyword)
      MINOR(Error_Symbol_or_string_package_attribute_bad_value)
      MINOR(Error_Symbol_or_string_package_attribute_missing)
      MINOR(Error_Symbol_or_string_directive_option)
  MAJOR(Error_Symbol_expected)
      MINOR(Error_Symbol_expected_user_defined)
      MINOR(Error_Symbol_expected_drop)
      MINOR(Error_Symbol_expected_expose)
      MINOR(Error_Symbol_expected_parse)
      MINOR(Error_Symbol_expected_var)
      MINOR(Error_Symbol_expected_numeric)
      MINOR(Error_Symbol_expected_varref)
      MINOR(Error_Symbol_expected_leave)
      MINOR(Error_Symbol_expected_iterate)
      MINOR(Error_Symbol_expected_end)
      MINOR(Error_Symbol_expected_on)
      MINOR(Error_Symbol_expected_off)
      MINOR(Error_Symbol_expected_use)
      MINOR(Error_Symbol_expected_raise)
      MINOR(Error_Symbol_expected_user)
      MINOR(Error_Symbol_expected_directive)
      MINOR(Error_Symbol_expected_colon)
      MINOR(Error_Symbol_expected_LABEL)
      MINOR(Error_Symbol_expected_annotation_attribute)
      MINOR(Error_Symbol_expected_namespace)
      MINOR(Error_Symbol_expected_namespace_class)
      MINOR(Error_Symbol_expected_qualified_call)
      MINOR(Error_Symbol_expected_qualified_symbol)
      MINOR(Error_Symbol_expected_annotation_type)
      MINOR(Error_Symbol_expected_form)
      MINOR(Error_Symbol_expected_delegate)
      MINOR(Error_Symbol_expected_use_local)
      MINOR(Error_Symbol_expected_indirect)
      MINOR(Error_Symbol_expected_after_keyword)
  MAJOR(Error_Invalid_data)
      MINOR(Error_Invalid_data_user_defined)
      MINOR(Error_Invalid_data_nop)
      MINOR(Error_Invalid_data_select)
      MINOR(Error_Invalid_data_name)
      MINOR(Error_Invalid_data_condition)
      MINOR(Error_Invalid_data_signal)
      MINOR(Error_Invalid_data_trace)
      MINOR(Error_Invalid_data_leave)
      MINOR(Error_Invalid_data_iterate)
      MINOR(Error_Invalid_data_end)
      MINOR(Error_Invalid_data_form)
      MINOR(Error_Invalid_data_guard_off)
      MINOR(Error_Invalid_data_constant_dir)
      MINOR(Error_Invalid_data_resource_dir)
  MAJOR(Error_Invalid_character_string)
      MINOR(Error_Invalid_character_string_char)
      MINOR(Error_Invalid_character_string_DBCS)
      MINOR(Error_Invalid_character_string_user_defined)
  MAJOR(Error_Invalid_data_string)
      MINOR(Error_Invalid_data_string_char)
      MINOR(Error_Invalid_data_string_user_defined)
  MAJOR(Error_Invalid_trace)
      MINOR(Error_Invalid_trace_trace)
      MINOR(Error_Invalid_trace_debug)
  MAJOR(Error_Invalid_subkeyword)
      MINOR(Error_Invalid_subkeyword_callon)
      MINOR(Error_Invalid_subkeyword_calloff)
      MINOR(Error_Invalid_subkeyword_signalon)
      MINOR(Error_Invalid_subkeyword_signaloff)
      MINOR(Error_Invalid_subkeyword_numeric)
      MINOR(Error_Invalid_subkeyword_form)
      MINOR(Error_Invalid_subkeyword_string_user_defined)
      MINOR(Error_Invalid_subkeyword_class)
      MINOR(Error_Invalid_subkeyword_method)
      MINOR(Error_Invalid_subkeyword_routine)
      MINOR(Error_Invalid_subkeyword_requires)
      MINOR(Error_Invalid_subkeyword_procedure)
      MINOR(Error_Invalid_subkeyword_callonname)
      MINOR(Error_Invalid_subkeyword_signalonname)
      MINOR(Error_Invalid_subkeyword_parse)
      MINOR(Error_Invalid_subkeyword_use)
      MINOR(Error_Invalid_subkeyword_use_strict)
      MINOR(Error_Invalid_subkeyword_raise)
      MINOR(Error_Invalid_subkeyword_raiseoption)
      MINOR(Error_Invalid_subkeyword_description)
      MINOR(Error_Invalid_subkeyword_additional)
      MINOR(Error_Invalid_subkeyword_result)
      MINOR(Error_Invalid_subkeyword_guard_on)
      MINOR(Error_Invalid_subkeyword_guard)
      MINOR(Error_Invalid_subkeyword_forward_option)
      MINOR(Error_Invalid_subkeyword_to)
      MINOR(Error_Invalid_subkeyword_arguments)
      MINOR(Error_Invalid_subkeyword_continue)
      MINOR(Error_Invalid_subkeyword_forward_class)
      MINOR(Error_Invalid_subkeyword_message)
      MINOR(Error_Invalid_subkeyword_select)
      MINOR(Error_Invalid_subkeyword_options)
      MINOR(Error_Invalid_subkeyword_attribute)
      MINOR(Error_Invalid_subkeyword_resource)
      MINOR(Error_Invalid_subkeyword_following)
      MINOR(Error_Invalid_subkeyword_annotation)
  MAJOR(Error_Invalid_whole_number)
      MINOR(Error_Invalid_whole_number_power)
      MINOR(Error_Invalid_whole_number_repeat)
      MINOR(Error_Invalid_whole_number_for)
      MINOR(Error_Invalid_whole_number_parse)
      MINOR(Error_Invalid_whole_number_digits)
      MINOR(Error_Invalid_whole_number_fuzz)
      MINOR(Error_Invalid_whole_number_trace)
      MINOR(Error_Invalid_whole_number_intdiv)
      MINOR(Error_Invalid_whole_number_rem)
      MINOR(Error_Invalid_whole_number_method)
      MINOR(Error_Invalid_whole_number_user_defined)
      MINOR(Error_Invalid_whole_number_compareto)
      MINOR(Error_Invalid_whole_number_compare)
  MAJOR(Error_Invalid_do)
      MINOR(Error_Invalid_do_whileuntil)
      MINOR(Error_Invalid_do_forever)
      MINOR(Error_Invalid_do_duplicate)
      MINOR(Error_Invalid_do_with_no_control)
      MINOR(Error_Invalid_do_with_no_over)
  MAJOR(Error_Invalid_leave)
      MINOR(Error_Invalid_leave_leave)
      MINOR(Error_Invalid_leave_iterate)
      MINOR(Error_Invalid_leave_leavevar)
      MINOR(Error_Invalid_leave_iteratevar)
      MINOR(Error_Invalid_leave_iterate_name)
  MAJOR(Error_Environment_name)
      MINOR(Error_Environment_name_name)
  MAJOR(Error_Name_too_long)
      MINOR(Error_Name_too_long_name)
      MINOR(Error_Name_too_long_string)
      MINOR(Error_Name_too_long_user_defined)
      MINOR(Error_Name_too_long_hex)
      MINOR(Error_Name_too_long_bin)
  MAJOR(Error_Invalid_variable)
      MINOR(Error_Invalid_variable_assign)
      MINOR(Error_Invalid_variable_number)
      MINOR(Error_Invalid_variable_period)
      MINOR(Error_Invalid_variable_user_defined)
  MAJOR(Error_Expression_result)
      MINOR(Error_Expression_result_digits)
      MINOR(Error_Expression_result_maxdigits)
      MINOR(Error_Expression_user_defined)
      MINOR(Error_Expression_result_address)
      MINOR(Error_Expression_result_signal)
      MINOR(Error_Expression_result_trace)
      MINOR(Error_Expression_result_raise)
  MAJOR(Error_Logical_value)
      MINOR(Error_Logical_value_if)
      MINOR(Error_Logical_value_when)
      MINOR(Error_Logical_value_while)
      MINOR(Error_Logical_value_until)
      MINOR(Error_Logical_value_logical)
      MINOR(Error_Logical_value_logical_list)
      MINOR(Error_Logical_value_user_defined)
      MINOR(Error_Logical_value_method)
      MINOR(Error_Logical_value_guard)
      MINOR(Error_Logical_value_authorization)
      MINOR(Error_Logical_value_property)
      MINOR(Error_Logical_value_when_case)
      MINOR(Error_Logical_value_supplier)
  MAJOR(Error_Invalid_expression)
      MINOR(Error_Invalid_expression_general)
      MINOR(Error_Invalid_expression_user_defined)
      MINOR(Error_Invalid_expression_prefix)
      MINOR(Error_Invalid_expression_if)
      MINOR(Error_Invalid_expression_when)
      MINOR(Error_Invalid_expression_control)
      MINOR(Error_Invalid_expression_by)
      MINOR(Error_Invalid_expression_to)
      MINOR(Error_Invalid_expression_for)
      MINOR(Error_Invalid_expression_while)
      MINOR(Error_Invalid_expression_until)
      MINOR(Error_Invalid_expression_over)
      MINOR(Error_Invalid_expression_interpret)
      MINOR(Error_Invalid_expression_options)
      MINOR(Error_Invalid_expression_address)
      MINOR(Error_Invalid_expression_signal)
      MINOR(Error_Invalid_expression_trace)
      MINOR(Error_Invalid_expression_form)
      MINOR(Error_Invalid_expression_assign)
      MINOR(Error_Invalid_expression_operator)
      MINOR(Error_Invalid_expression_guard)
      MINOR(Error_Invalid_expression_raise_description)
      MINOR(Error_Invalid_expression_raise_additional)
      MINOR(Error_Invalid_expression_raise_list)
      MINOR(Error_Invalid_expression_forward_to)
      MINOR(Error_Invalid_expression_forward_arguments)
      MINOR(Error_Invalid_expression_forward_message)
      MINOR(Error_Invalid_expression_forward_class)
      MINOR(Error_Invalid_expression_logical_list)
      MINOR(Error_Invalid_expression_use_arg_default)
      MINOR(Error_Invalid_expression_parse)
      MINOR(Error_Invalid_expression_call)
      MINOR(Error_Invalid_expression_select_case)
      MINOR(Error_Invalid_expression_case_when_list)
  MAJOR(Error_Unmatched_parenthesis)
      MINOR(Error_Unmatched_parenthesis_user_defined)
      MINOR(Error_Unmatched_parenthesis_paren)
      MINOR(Error_Unmatched_parenthesis_square)
  MAJOR(Error_Unexpected_comma)
      MINOR(Error_Unexpected_comma_comma)
      MINOR(Error_Unexpected_comma_paren)
      MINOR(Error_Unexpected_comma_user_defined)
      MINOR(Error_Unexpected_comma_bracket)
  MAJOR(Error_Invalid_template)
      MINOR(Error_Invalid_template_trigger)
      MINOR(Error_Invalid_template_position)
      MINOR(Error_Invalid_template_with)
      MINOR(Error_Invalid_template_user_defined)
      MINOR(Error_Invalid_template_missing)
  MAJOR(Error_Evaluation_stack_overflow)
  MAJOR(Error_Incorrect_call)
      MINOR(Error_Incorrect_call_external)
      MINOR(Error_Incorrect_call_result)
      MINOR(Error_Incorrect_call_minarg)
      MINOR(Error_Incorrect_call_maxarg)
      MINOR(Error_Incorrect_call_noarg)
      MINOR(Error_Incorrect_call_number)
      MINOR(Error_Incorrect_call_whole)
      MINOR(Error_Incorrect_call_nonnegative)
      MINOR(Error_Incorrect_call_positive)
      MINOR(Error_Incorrect_call_toobig)
      MINOR(Error_Incorrect_call_range)
      MINOR(Error_Incorrect_call_null)
      MINOR(Error_Incorrect_call_option)
      MINOR(Error_Incorrect_call_pad)
      MINOR(Error_Incorrect_call_binary)
      MINOR(Error_Incorrect_call_hex)
      MINOR(Error_Incorrect_call_symbol)
      MINOR(Error_Incorrect_call_pad_or_name)
      MINOR(Error_Incorrect_call_list)
      MINOR(Error_Incorrect_call_trace)
      MINOR(Error_Incorrect_call_random)
      MINOR(Error_Incorrect_call_sourceline)
      MINOR(Error_Incorrect_call_x2d)
      MINOR(Error_Incorrect_call_format_invalid)
      MINOR(Error_Incorrect_call_invalid_conversion)
      MINOR(Error_Incorrect_call_user_defined)
      MINOR(Error_Incorrect_call_random_range)
      MINOR(Error_Incorrect_call_stream_name)
      MINOR(Error_Incorrect_call_array)
      MINOR(Error_Incorrect_call_nostring)
      MINOR(Error_Incorrect_call_selector)
      MINOR(Error_Incorrect_call_parm_wrong_sep)
      MINOR(Error_Incorrect_call_format_incomp_sep)
      MINOR(Error_Incorrect_call_queue_no_char)
      MINOR(Error_Incorrect_call_read_from_writeonly)
      MINOR(Error_Incorrect_call_write_to_readonly)
      MINOR(Error_Incorrect_call_signature)
      MINOR(Error_Incorrect_call_nostem)
  MAJOR(Error_Conversion)
      MINOR(Error_Conversion_operator)
      MINOR(Error_Conversion_prefix)
      MINOR(Error_Conversion_exponent)
      MINOR(Error_Conversion_to)
      MINOR(Error_Conversion_by)
      MINOR(Error_Conversion_control)
      MINOR(Error_Conversion_user_defined)
      MINOR(Error_Conversion_raise)
  MAJOR(Error_Overflow)
      MINOR(Error_Overflow_overflow)
      MINOR(Error_Overflow_underflow)
      MINOR(Error_Overflow_zero)
      MINOR(Error_Overflow_user_defined)
      MINOR(Error_Overflow_expoverflow)
      MINOR(Error_Overflow_expunderflow)
      MINOR(Error_Overflow_power)
  MAJOR(Error_Routine_not_found)
      MINOR(Error_Routine_not_found_name)
      MINOR(Error_Routine_not_found_requires)
      MINOR(Error_Routine_not_found_user_defined)
      MINOR(Error_Routine_not_found_namespace)
  MAJOR(Error_Function_no_data)
      MINOR(Error_Function_no_data_function)
      MINOR(Error_Function_no_data_user_defined)
  MAJOR(Error_No_data_on_return)
      MINOR(Error_No_data_on_return_name)
  MAJOR(Error_Variable_reference)
      MINOR(Error_Variable_reference_extra)
      MINOR(Error_Variable_reference_user_defined)
      MINOR(Error_Variable_reference_missing)
      MINOR(Error_Variable_reference_use)
  MAJOR(Error_Unexpected_label)
      MINOR(Error_Unexpected_label_interpret)
  MAJOR(Error_System_service)
      MINOR(Error_System_service_service)
      MINOR(Error_System_service_user_defined)
  MAJOR(Error_Interpretation)
      MINOR(Error_Interpretation_initialization)
      MINOR(Error_Interpretation_switch)
      MINOR(Error_Interpretation_user_defined)
  MAJOR(Error_Invalid_argument)
      MINOR(Error_Invalid_argument_user_defined)
      MINOR(Error_Invalid_argument_noarg)
      MINOR(Error_Invalid_argument_number)
      MINOR(Error_Invalid_argument_whole)
      MINOR(Error_Invalid_argument_nonnegative)
      MINOR(Error_Invalid_argument_positive)
      MINOR(Error_Invalid_argument_toobig)
      MINOR(Error_Invalid_argument_range)
      MINOR(Error_Invalid_argument_null)
      MINOR(Error_Invalid_argument_string)
      MINOR(Error_Invalid_argument_pad)
      MINOR(Error_Invalid_argument_length)
      MINOR(Error_Invalid_argument_position)
      MINOR(Error_Invalid_argument_noarray)
      MINOR(Error_Invalid_argument_noclass)
      MINOR(Error_Invalid_argument_argType)
      MINOR(Error_Invalid_argument_list)
      MINOR(Error_Invalid_argument_general)
      MINOR(Error_Invalid_argument_format)
      MINOR(Error_Invalid_argument_pointer)
      MINOR(Error_Invalid_argument_nostem)
      MINOR(Error_Invalid_argument_double)
      MINOR(Error_Invalid_argument_maxarg)
      MINOR(Error_Invalid_argument_array)
      MINOR(Error_Invalid_argument_array_size)
      MINOR(Error_Invalid_argument_nonnegative_number)
      MINOR(Error_Invalid_argument_positive_number)
      MINOR(Error_Invalid_argument_logical)
  MAJOR(Error_Variable_expected)
      MINOR(Error_Variable_expected_USE)
      MINOR(Error_Variable_expected_PARSE)
  MAJOR(Error_External_name_not_found)
      MINOR(Error_External_name_not_found_user_defined)
      MINOR(Error_External_name_not_found_class)
      MINOR(Error_External_name_not_found_method)
      MINOR(Error_External_name_not_found_routine)
  MAJOR(Error_No_result_object)
      MINOR(Error_No_result_object_user_defined)
      MINOR(Error_No_result_object_message)
  MAJOR(Error_OLE_Error)
      MINOR(Error_OLE_Error_user_defined)
      MINOR(Error_Unknown_OLE_Error)
      MINOR(Error_Variant2Rexx)
      MINOR(Error_Rexx2Variant)
      MINOR(Error_Argument_Count_Mismatch)
      MINOR(Error_Invalid_Variant)
      MINOR(Error_OLE_Exception)
      MINOR(Error_Unknown_OLE_Method)
      MINOR(Error_Coercion_Failed_Overflow)
      MINOR(Error_Coercion_Failed_Type_Mismatch)
      MINOR(Error_Parameter_Omitted)
      MINOR(Error_No_OLE_instance)
      MINOR(Error_Client_Disconnected_From_Server)
  MAJOR(Error_Incorrect_method)
      MINOR(Error_Incorrect_method_user_defined)
      MINOR(Error_Incorrect_method_minarg)
      MINOR(Error_Incorrect_method_stream_type)
      MINOR(Error_Incorrect_method_maxarg)
      MINOR(Error_Incorrect_method_noarg)
      MINOR(Error_Incorrect_method_number)
      MINOR(Error_Incorrect_method_whole)
      MINOR(Error_Incorrect_method_nonnegative)
      MINOR(Error_Incorrect_method_positive)
      MINOR(Error_Incorrect_method_toobig)
      MINOR(Error_Incorrect_method_range)
      MINOR(Error_Incorrect_method_null)
      MINOR(Error_Incorrect_method_hex)
      MINOR(Error_Incorrect_method_symbol)
      MINOR(Error_Incorrect_method_list)
      MINOR(Error_Incorrect_method_option)
      MINOR(Error_Incorrect_method_string)
      MINOR(Error_Incorrect_method_methodname)
      MINOR(Error_Incorrect_method_index)
      MINOR(Error_Incorrect_method_array)
      MINOR(Error_Incorrect_method_binary)
      MINOR(Error_Incorrect_method_pad)
      MINOR(Error_Incorrect_method_length)
      MINOR(Error_Incorrect_method_position)
      MINOR(Error_Incorrect_method_minsub)
      MINOR(Error_Incorrect_method_maxsub)
      MINOR(Error_Incorrect_method_d2xd2c)
      MINOR(Error_Incorrect_method_d2x)
      MINOR(Error_Incorrect_method_d2c)
      MINOR(Error_Incorrect_method_hexblank)
      MINOR(Error_Incorrect_method_binblank)
      MINOR(Error_Incorrect_method_invhex)
      MINOR(Error_Incorrect_method_invbin)
      MINOR(Error_Incorrect_method_x2dbig)
      MINOR(Error_Incorrect_method_c2dbig)
      MINOR(Error_Incorrect_method_supplier)
      MINOR(Error_Incorrect_method_nostring)
      MINOR(Error_Incorrect_method_noarray)
      MINOR(Error_Incorrect_method_string_no_whole_number)
      MINOR(Error_Incorrect_method_exponent_oversize)
      MINOR(Error_Incorrect_method_before_oversize)
      MINOR(Error_Incorrect_method_string_nonumber)
      MINOR(Error_Incorrect_method_nomessage)
      MINOR(Error_Incorrect_method_message_noarg)
      MINOR(Error_Incorrect_method_message)
      MINOR(Error_Incorrect_method_section)
      MINOR(Error_Incorrect_method_noclass)
      MINOR(Error_Incorrect_method_nomatch)
      MINOR(Error_Incorrect_method_time)
      MINOR(Error_Incorrect_method_no_method)
      MINOR(Error_Incorrect_method_argType)
      MINOR(Error_Incorrect_method_array_dimension)
      MINOR(Error_Incorrect_method_nostring_inarray)
      MINOR(Error_Incorrect_method_array_nostring)
      MINOR(Error_Incorrect_method_array_noclass)
      MINOR(Error_Incorrect_method_array_too_big)
      MINOR(Error_Incorrect_method_invbase64)
      MINOR(Error_Unsupported_method)
      MINOR(Error_Application_error)
      MINOR(Error_Incorrect_method_abstract)
      MINOR(Error_Incorrect_method_queue_index)
      MINOR(Error_Unsupported_new_method)
      MINOR(Error_Incorrect_method_signature)
      MINOR(Error_Incorrect_method_nostem)
      MINOR(Error_Unsupported_copy_method)
      MINOR(Error_Incorrect_method_multi_dimension)
      MINOR(Error_Incorrect_method_message_name)
      MINOR(Error_Incorrect_method_no_method_type)
      MINOR(Error_Incorrect_method_nil_not_orderable)
  MAJOR(Error_No_method)
      MINOR(Error_No_method_name)
      MINOR(Error_No_method_user_defined)
  MAJOR(Error_Execution)
      MINOR(Error_Execution_user_defined)
      MINOR(Error_Execution_nodouble)
      MINOR(Error_Execution_library)
      MINOR(Error_Execution_terminate)
      MINOR(Error_Execution_deadlock)
      MINOR(Error_Execution_badobject)
      MINOR(Error_Execution_wrongobject)
      MINOR(Error_Execution_nometaclass)
      MINOR(Error_Execution_noclass)
      MINOR(Error_Execution_cyclic)
      MINOR(Error_Execution_noarray)
      MINOR(Error_Execution_nostring)
      MINOR(Error_Execution_message_reuse)
      MINOR(Error_Execution_message_error)
      MINOR(Error_Execution_raise_object)
      MINOR(Error_Execution_propagate)
      MINOR(Error_Execution_nomethod)
      MINOR(Error_Execution_reply)
      MINOR(Error_Execution_reply_return)
      MINOR(Error_Execution_reply_exit)
      MINOR(Error_Execution_super)
      MINOR(Error_Execution_syntax_additional)
      MINOR(Error_Execution_error_condition)
      MINOR(Error_Execution_mixinclass)
      MINOR(Error_Execution_baseclass)
      MINOR(Error_Execution_recursive_inherit)
      MINOR(Error_Execution_uninherit)
      MINOR(Error_Execution_forward_arguments)
      MINOR(Error_Execution_forward)
      MINOR(Error_Execution_authorization)
      MINOR(Error_Execution_no_concurrency)
      MINOR(Error_Execution_sparse_array)
      MINOR(Error_Execution_nostem)
      MINOR(Error_Execution_library_method)
      MINOR(Error_Execution_library_routine)
      MINOR(Error_Execution_native_routine)
      MINOR(Error_Execution_context_not_active)
      MINOR(Error_Execution_library_version)
      MINOR(Error_Execution_invalid_thread)
      MINOR(Error_Execution_rexx_package_update)
      MINOR(Error_Execution_rexx_defined_class)
      MINOR(Error_Execution_unassigned_variable)
      MINOR(Error_Execution_no_namespace)
      MINOR(Error_Execution_no_namespace_class)
      MINOR(Error_Execution_abstract_class)
      MINOR(Error_Execution_abstract_metaclass)
      MINOR(Error_Execution_private_access)
      MINOR(Error_Execution_expose_method)
      MINOR(Error_Execution_use_local_method)
      MINOR(Error_Execution_no_supplier)
  MAJOR(Error_Translation)
      MINOR(Error_Translation_user_defined)
      MINOR(Error_Translation_duplicate_class)
      MINOR(Error_Translation_duplicate_method)
      MINOR(Error_Translation_duplicate_routine)
      MINOR(Error_Translation_duplicate_requires)
      MINOR(Error_Translation_missing_class)
      MINOR(Error_Translation_bad_metaclass)
      MINOR(Error_Translation_expose)
      MINOR(Error_Translation_use_local)
      MINOR(Error_Translation_expose_interpret)
      MINOR(Error_Translation_guard)
      MINOR(Error_Translation_guard_guard)
      MINOR(Error_Translation_guard_interpret)
      MINOR(Error_Translation_guard_expose)
      MINOR(Error_Translation_directive_interpret)
      MINOR(Error_Translation_use_local_interpret)
      MINOR(Error_Translation_bad_directive)
      MINOR(Error_Translation_bad_external)
      MINOR(Error_Translation_use_comma)
      MINOR(Error_Translation_reply)
      MINOR(Error_Translation_invalid_line)
      MINOR(Error_Translation_requires)
      MINOR(Error_Translation_reply_interpret)
      MINOR(Error_Translation_forward_interpret)
      MINOR(Error_Translation_invalid_attribute)
      MINOR(Error_Translation_class_external_bad_parameters)
      MINOR(Error_Translation_class_external_bad_class_name)
      MINOR(Error_Translation_class_external_bad_class_server)
      MINOR(Error_Translation_use_arg_ellipsis)
      MINOR(Error_Translation_duplicate_attribute)
      MINOR(Error_Translation_duplicate_constant)
      MINOR(Error_Translation_abstract_method)
      MINOR(Error_Translation_attribute_method)
      MINOR(Error_Translation_external_attribute)
      MINOR(Error_Translation_external_method)
      MINOR(Error_Translation_body_error)
      MINOR(Error_Translation_constant_body)
      MINOR(Error_Translation_external_routine)
      MINOR(Error_Translation_abstract_attribute)
      MINOR(Error_Translation_directive_method_routine)
      MINOR(Error_Translation_duplicate_resource)
      MINOR(Error_Translation_missing_resource_end)
      MINOR(Error_Translation_reserved_namespace)
      MINOR(Error_Translation_missing_annotation_target)
      MINOR(Error_Translation_delegate_method)
      MINOR(Error_Translation_delegate_attribute)
      MINOR(Error_Translation_use_local_compound)
      MINOR(Error_Translation_bad_class)
  MAJOR(Error_at_line)
      MINOR(Message_Translations_error)
      MINOR(Message_Translations_running)
      MINOR(Message_Translations_line)
      MINOR(Message_Translations_debug_error)
      MINOR(Message_Translations_debug_prompt)
      MINOR(Message_Translations_January)
      MINOR(Message_Translations_February)
      MINOR(Message_Translations_March)
      MINOR(Message_Translations_April)
      MINOR(Message_Translations_May)
      MINOR(Message_Translations_June)
      MINOR(Message_Translations_July)
      MINOR(Message_Translations_August)
      MINOR(Message_Translations_September)
      MINOR(Message_Translations_October)
      MINOR(Message_Translations_November)
      MINOR(Message_Translations_December)
      MINOR(Message_Translations_routine_invocation)
      MINOR(Message_Translations_method_invocation)
      MINOR(Message_Translations_compiled_method_invocation)
      MINOR(Message_Translations_compiled_routine_invocation)
      MINOR(Message_Translations_no_source_available)
      MINOR(Message_Translations_internal_code)
      MINOR(Message_Translations_sourceless_method_invocation)
      MINOR(Message_Translations_sourceless_routine_invocation)
      MINOR(Message_Translations_sourceless_program_invocation)

  MAJOR(Table_end)                       /* make sure table is ended          */
};
#endif

/* -------------------------------------------------------------------------- */
/* --            file is generated by build process                        -- */
/* --            ==================================================        -- */
/* --            DO NOT CHANGE THIS FILE, ALL CHANGES WILL BE LOST!        -- */
/* --            ==================================================        -- */
/* -------------------------------------------------------------------------- */
