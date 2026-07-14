#include "tfhe.h"

#include <assert.h>
#include <inttypes.h>
#include <stdio.h>

int client_key_test(const ClientKey *client_key) {
  int ok;
  FheBool *lhs = NULL;
  FheBool *rhs = NULL;
  FheBool *result = NULL;

  bool lhs_clear = 0;
  bool rhs_clear = 1;

  ok = fhe_bool_try_encrypt_with_client_key_bool(lhs_clear, client_key, &lhs);
  assert(ok == 0);

  ok = fhe_bool_try_encrypt_with_client_key_bool(rhs_clear, client_key, &rhs);
  assert(ok == 0);

  ok = fhe_bool_bitand(lhs, rhs, &result);
  assert(ok == 0);

  bool clear;
  ok = fhe_bool_decrypt(result, client_key, &clear);
  assert(ok == 0);

  assert(clear == (lhs_clear & rhs_clear));

  fhe_bool_destroy(lhs);
  fhe_bool_destroy(rhs);
  fhe_bool_destroy(result);

  return ok;
}

int public_key_test(const ClientKey *client_key, const PublicKey *public_key) {
  int ok;
  FheBool *lhs = NULL;
  FheBool *rhs = NULL;
  FheBool *result = NULL;

  bool lhs_clear = 0;
  bool rhs_clear = 1;

  ok = fhe_bool_try_encrypt_with_public_key_bool(lhs_clear, public_key, &lhs);
  assert(ok == 0);

  ok = fhe_bool_try_encrypt_with_public_key_bool(rhs_clear, public_key, &rhs);
  assert(ok == 0);

  ok = fhe_bool_bitand(lhs, rhs, &result);
  assert(ok == 0);

  bool clear;
  ok = fhe_bool_decrypt(result, client_key, &clear);
  assert(ok == 0);

  assert(clear == (lhs_clear & rhs_clear));

  fhe_bool_destroy(lhs);
  fhe_bool_destroy(rhs);
  fhe_bool_destroy(result);

  return ok;
}

int trivial_encrypt_test(const ClientKey *client_key) {
  int ok;
  FheBool *lhs = NULL;
  FheBool *rhs = NULL;
  FheBool *result = NULL;

  bool lhs_clear = 0;
  bool rhs_clear = 1;

  ok = fhe_bool_try_encrypt_trivial_bool(lhs_clear, &lhs);
  assert(ok == 0);

  ok = fhe_bool_try_encrypt_trivial_bool(rhs_clear, &rhs);
  assert(ok == 0);

  ok = fhe_bool_bitand(lhs, rhs, &result);
  assert(ok == 0);

  bool clear;
  ok = fhe_bool_decrypt(result, client_key, &clear);
  assert(ok == 0);

  assert(clear == (lhs_clear & rhs_clear));

  fhe_bool_destroy(lhs);
  fhe_bool_destroy(rhs);
  fhe_bool_destroy(result);

  return ok;
}

int bool_if_then_else_test(const ClientKey *client_key) {
  int ok;
  FheBool *cond = NULL;
  FheBool *then_ct = NULL;
  FheBool *else_ct = NULL;
  FheBool *result = NULL;

  ok = fhe_bool_try_encrypt_with_client_key_bool(true, client_key, &cond);
  assert(ok == 0);
  ok = fhe_bool_try_encrypt_with_client_key_bool(true, client_key, &then_ct);
  assert(ok == 0);
  ok = fhe_bool_try_encrypt_with_client_key_bool(false, client_key, &else_ct);
  assert(ok == 0);

  // Test true condition
  ok = fhe_bool_if_then_else(cond, then_ct, else_ct, &result);
  assert(ok == 0);

  bool clear;
  ok = fhe_bool_decrypt(result, client_key, &clear);
  assert(ok == 0);
  assert(clear == true);

  fhe_bool_destroy(result);
  result = NULL;

  // Test false condition
  fhe_bool_destroy(cond);
  cond = NULL;
  ok = fhe_bool_try_encrypt_with_client_key_bool(false, client_key, &cond);
  assert(ok == 0);

  ok = fhe_bool_if_then_else(cond, then_ct, else_ct, &result);
  assert(ok == 0);

  ok = fhe_bool_decrypt(result, client_key, &clear);
  assert(ok == 0);
  assert(clear == false);

  fhe_bool_destroy(cond);
  fhe_bool_destroy(then_ct);
  fhe_bool_destroy(else_ct);
  fhe_bool_destroy(result);

  return ok;
}

int main(void) {

  ConfigBuilder *builder;
  Config *config;

  config_builder_default(&builder);
  config_builder_build(builder, &config);

  ClientKey *client_key = NULL;
  ServerKey *server_key = NULL;
  PublicKey *public_key = NULL;

  generate_keys(config, &client_key, &server_key);
  public_key_new(client_key, &public_key);

  set_server_key(server_key);

  client_key_test(client_key);
  public_key_test(client_key, public_key);
  trivial_encrypt_test(client_key);
  bool_if_then_else_test(client_key);

  client_key_destroy(client_key);
  public_key_destroy(public_key);
  server_key_destroy(server_key);
  unset_server_key();

  return EXIT_SUCCESS;
}
