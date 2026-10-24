#include <freemvvm/abi.h>

#include <stdlib.h>

typedef struct command_probe {
  ui_handle runtime;
  ui_handle retained_cancel;
  uint64_t operation_id;
  uint32_t start_count;
  uint32_t destroy_count;
  ui_status completion_status;
} command_probe;

typedef struct terminal_post {
  command_probe *probe;
  ui_handle command;
  uint64_t operation_id;
} terminal_post;

static void wake(void *userdata, uint32_t priority) {
  (void)userdata;
  (void)priority;
}

static void bridge_destroy(void *userdata) { (void)userdata; }

static void terminal_callback(void *userdata) {
  terminal_post *post = (terminal_post *)userdata;
  post->probe->completion_status = ui_abi_1_command_complete_success(
      post->command, post->operation_id);
}

static void terminal_destroy(void *userdata) {
  terminal_post *post = (terminal_post *)userdata;
  (void)ui_abi_1_handle_release(post->command);
  free(post);
}

static ui_status command_start(void *userdata, ui_handle command,
                               uint64_t operation_id,
                               ui_handle cancel_token) {
  command_probe *probe = (command_probe *)userdata;
  ui_handle retained_command = 0;
  ui_status status = ui_abi_1_handle_retain(command, &retained_command);
  if (status != UI_OK) {
    return status;
  }
  status = ui_abi_1_handle_retain(cancel_token, &probe->retained_cancel);
  if (status != UI_OK) {
    (void)ui_abi_1_handle_release(retained_command);
    return status;
  }
  probe->operation_id = operation_id;
  probe->start_count++;

  terminal_post *post = (terminal_post *)malloc(sizeof(*post));
  if (post == NULL) {
    (void)ui_abi_1_handle_release(retained_command);
    (void)ui_abi_1_handle_release(probe->retained_cancel);
    probe->retained_cancel = 0;
    return UI_OUT_OF_MEMORY;
  }
  post->probe = probe;
  post->command = retained_command;
  post->operation_id = operation_id;
  status = ui_abi_1_runtime_post(probe->runtime, terminal_callback, post,
                                 terminal_destroy, NULL);
  if (status != UI_OK) {
    terminal_destroy(post);
    return status;
  }
  return UI_OK;
}

static void command_destroy(void *userdata) {
  command_probe *probe = (command_probe *)userdata;
  probe->destroy_count++;
  if (probe->retained_cancel != 0) {
    (void)ui_abi_1_handle_release(probe->retained_cancel);
    probe->retained_cancel = 0;
  }
}

int main(void) {
  ui_handle runtime = 0;
  ui_handle eligibility = 0;
  ui_handle command = 0;
  ui_handle state_property = 0;
  ui_handle can_execute_property = 0;
  ui_handle error = 0;
  uint64_t scope = 0;
  uint64_t shutdown_request = 0;
  uint64_t operation_id = 0;
  uint64_t error_snapshot = 0;
  int64_t state = -1;
  ui_bool can_execute = 0;
  command_probe probe = {0};

  if (ui_abi_1_runtime_create(&runtime, &error) != UI_OK || runtime == 0 ||
      error != 0) {
    return 1;
  }
  probe.runtime = runtime;
  if (ui_abi_1_runtime_set_ui_loop_bridge(runtime, wake, NULL, bridge_destroy,
                                          NULL) != UI_OK ||
      ui_abi_1_property_create_bool(runtime, 1, &eligibility, &error) != UI_OK ||
      ui_abi_1_command_create_async(runtime, eligibility, command_start, &probe,
                                    command_destroy, &command, &error) != UI_OK ||
      command == 0 || error != 0 ||
      ui_abi_1_command_state_property(command, &state_property) != UI_OK ||
      ui_abi_1_command_can_execute_property(command, &can_execute_property) !=
          UI_OK) {
    return 2;
  }
  if (ui_abi_1_command_execute(command, &operation_id, &error) != UI_OK ||
      operation_id == 0 || error != 0 || probe.start_count != 1 ||
      probe.operation_id != operation_id ||
      ui_abi_1_property_get_i64(state_property, &state) != UI_OK || state != 1 ||
      ui_abi_1_property_get_bool(can_execute_property, &can_execute) != UI_OK ||
      can_execute != 0 ||
      ui_abi_1_runtime_dispatch_turn(runtime) != UI_OK ||
      probe.completion_status != UI_OK ||
      ui_abi_1_property_get_i64(state_property, &state) != UI_OK || state != 3 ||
      ui_abi_1_property_get_bool(can_execute_property, &can_execute) != UI_OK ||
      can_execute != 1 ||
      ui_abi_1_command_failure_snapshot(command, &error_snapshot) != UI_OK ||
      error_snapshot != 0 ||
      ui_abi_1_command_close(command, &error) != UI_OK || error != 0 ||
      probe.destroy_count != 1) {
    return 3;
  }

  if (ui_abi_1_runtime_host_begin_event(runtime, &scope) != UI_OK ||
      ui_abi_1_runtime_request_shutdown(runtime, UI_SHUTDOWN_ASYNC,
                                        &shutdown_request, &error) != UI_PENDING ||
      error != 0 || ui_abi_1_runtime_host_end_event(runtime, scope) != UI_OK ||
      ui_abi_1_handle_release(shutdown_request) != UI_OK ||
      ui_abi_1_handle_release(command) != UI_OK ||
      ui_abi_1_handle_release(state_property) != UI_OK ||
      ui_abi_1_handle_release(can_execute_property) != UI_OK ||
      ui_abi_1_handle_release(eligibility) != UI_OK ||
      ui_abi_1_handle_release(runtime) != UI_OK) {
    return 4;
  }
  return 0;
}
