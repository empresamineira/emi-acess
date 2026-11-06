import 'package:flutter_hbb/desktop/widgets/emi_server_gate.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('only a well-formed success unlocks', () {
    final unlocked = parseEmiGateResponse(
        '{"ok":true,"configured":true,"retry_after":0}');
    expect(unlocked.ok, isTrue);
    expect(emiGateErrorText(unlocked), isNull);

    for (final response in ['', 'error', '{}', '{"ok":true}', '[]']) {
      expect(parseEmiGateResponse(response).ok, isFalse, reason: response);
    }
  });

  test('tells the user why access was denied', () {
    String? errorFor(String response) =>
        emiGateErrorText(parseEmiGateResponse(response));

    expect(errorFor('{"ok":false,"configured":true,"retry_after":0}'),
        'Senha incorreta.');
    expect(errorFor('{"ok":false,"configured":true,"retry_after":300}'),
        'Muitas tentativas. Tente novamente em 5 min.');
    expect(errorFor('{"ok":false,"configured":false,"retry_after":0}'),
        'A senha de acesso não foi configurada neste build.');
  });
}
