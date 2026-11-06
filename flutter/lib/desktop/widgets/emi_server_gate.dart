import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_hbb/common.dart';
import 'package:flutter_hbb/common/widgets/dialog.dart';
import 'package:flutter_hbb/models/platform_model.dart';
import 'package:get/get.dart';

class EmiGateResult {
  final bool ok;
  final bool configured;
  final int retryAfter;

  const EmiGateResult({
    required this.ok,
    required this.configured,
    required this.retryAfter,
  });

  static const denied =
      EmiGateResult(ok: false, configured: true, retryAfter: 0);
}

/// Anything other than a well-formed success counts as denied.
EmiGateResult parseEmiGateResponse(String response) {
  try {
    final json = jsonDecode(response);
    return EmiGateResult(
      ok: json['ok'] == true,
      configured: json['configured'] == true,
      retryAfter: json['retry_after'] as int,
    );
  } catch (_) {
    return EmiGateResult.denied;
  }
}

String? emiGateErrorText(EmiGateResult result) {
  if (result.ok) return null;
  if (!result.configured) {
    return 'A senha de acesso não foi configurada neste build.';
  }
  if (result.retryAfter > 0) {
    final minutes = (result.retryAfter / 60).ceil();
    return 'Muitas tentativas. Tente novamente em $minutes min.';
  }
  return 'Senha incorreta.';
}

EmiGateResult _verify(String password) {
  try {
    return parseEmiGateResponse(
        bind.mainGetCommonSync(key: 'emi-verify-server-settings:$password'));
  } catch (_) {
    return EmiGateResult.denied;
  }
}

void showEmiServerGate(VoidCallback onUnlocked) {
  final controller = TextEditingController();
  String? errorText;
  gFFI.dialogManager.show((setState, close, context) {
    submit() {
      final result = _verify(controller.text);
      controller.clear();
      if (result.ok) {
        close();
        onUnlocked();
        return;
      }
      setState(() {
        errorText = emiGateErrorText(result);
      });
    }

    return CustomAlertDialog(
      title: Text(translate('ID/Relay Server')),
      content: Row(
        children: [
          Expanded(
              child: PasswordWidget(
            title: 'Password',
            controller: controller,
            errorText: errorText,
            hintText: '',
          ))
        ],
      ).marginOnly(bottom: 12),
      actions: [
        dialogButton(translate('Cancel'), onPressed: close, isOutline: true),
        dialogButton(translate('OK'), onPressed: submit),
      ],
      onSubmit: submit,
      onCancel: close,
    );
  });
}
