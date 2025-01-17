#ifndef BRIDGESTANR_H
#define BRIDGESTANR_H

#ifdef __cplusplus
#include "model_rng.hpp"
extern "C" {
#else
typedef struct bs_model bs_model;
typedef struct bs_rng bs_rng;
typedef int bool;
#endif

// Shim to convert to R interface requirement of void with pointer args
// All calls directly delegated to versions without _R suffix
void bs_model_construct_R(char** data, int* rng, bs_model** ptr_out,
                          char** err_msg, void** err_ptr);

void bs_version_R(int* major, int* minor, int* patch);

void bs_model_destruct_R(bs_model** model);

/**
 * Free error message allocated in C++. Because R performs copies
 * at the boundary on char**s, this uses void** pointing to the same memory
 */
void bs_free_error_msg_R(void** err_msg);

void bs_name_R(bs_model** model, char const** name_out);

void bs_model_info_R(bs_model** model, char const** info_out);

void bs_param_names_R(bs_model** model, int* include_tp, int* include_gq,
                      char const** name_out);

void bs_param_unc_names_R(bs_model** model, char const** name_out);

void bs_param_num_R(bs_model** model, int* include_tp, int* include_gq,
                    int* num_out);

void bs_param_unc_num_R(bs_model** model, int* num_out);

void bs_param_constrain_R(bs_model** model, int* include_tp, int* include_gq,
                          const double* theta_unc, double* theta, bs_rng** rng,
                          int* return_code, char** err_msg, void** err_ptr);

void bs_param_unconstrain_R(bs_model** model, const double* theta,
                            double* theta_unc, int* return_code, char** err_msg,
                            void** err_ptr);

void bs_param_unconstrain_json_R(bs_model** model, char const** json,
                                 double* theta_unc, int* return_code,
                                 char** err_msg, void** err_ptr);

void bs_log_density_R(bs_model** model, int* propto, int* jacobian,
                      const double* theta_unc, double* val, int* return_code,
                      char** err_msg, void** err_ptr);

void bs_log_density_gradient_R(bs_model** model, int* propto, int* jacobian,
                               const double* theta_unc, double* val,
                               double* grad, int* return_code, char** err_msg,
                               void** err_ptr);

void bs_log_density_hessian_R(bs_model** model, int* propto, int* jacobian,
                              const double* theta_unc, double* val,
                              double* grad, double* hess, int* return_code,
                              char** err_msg, void** err_ptr);

void bs_rng_construct_R(int* seed, bs_rng** ptr_out, char** err_msg,
                        void** err_ptr);

void bs_rng_destruct_R(bs_rng** rng);

#ifdef __cplusplus
}
#endif

#endif
