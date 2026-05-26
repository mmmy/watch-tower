import importlib.util
import unittest
from pathlib import Path
from unittest.mock import patch


SCRIPT_PATH = Path(__file__).with_name("codex-wecom-signal.py")


def load_module():
    spec = importlib.util.spec_from_file_location("codex_wecom_signal", SCRIPT_PATH)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


class CodexWecomSignalTests(unittest.TestCase):
    def test_load_env_reads_wecom_bot_key_from_runtime_dir(self):
        module = load_module()
        runtime_dir = Path(__file__).parent / ".tmp-test-runtime"
        runtime_dir.mkdir(exist_ok=True)
        env_path = runtime_dir / ".env"
        env_path.write_text("WECOM_BOT_KEY=test-key\n", encoding="utf-8")

        try:
            values = module.load_env(runtime_dir)
        finally:
            env_path.unlink(missing_ok=True)
            runtime_dir.rmdir()

        self.assertEqual(values["WECOM_BOT_KEY"], "test-key")

    def test_build_codex_prompt_embeds_compact_alert_json(self):
        module = load_module()
        alert = {
            "symbol": "BTCUSDT",
            "groupName": "BTC Main",
            "period": "15",
            "signalType": "divMacd",
            "side": 1,
            "triggerTime": 1710000000000,
            "level": "normal",
        }

        prompt = module.build_codex_prompt(alert)

        self.assertTrue(prompt.startswith("/sma-trend 新信号触发:{"))
        self.assertIn('"symbol":"BTCUSDT"', prompt)
        self.assertIn('"period":"15"', prompt)
        self.assertTrue(prompt.endswith("请分析是否可以在本级别接下来5根k插针入场"))

    def test_resolve_codex_command_prefers_cmd_shim_on_windows(self):
        module = load_module()

        with patch.object(module.os, "name", "nt"), patch.object(
            module.shutil,
            "which",
            side_effect=lambda name: f"C:\\tools\\{name}" if name == "codex.cmd" else None,
        ):
            command = module.resolve_codex_command()

        self.assertEqual(command, "C:\\tools\\codex.cmd")

    def test_signal_type_filter_allows_only_configured_names(self):
        module = load_module()
        env = {"SIGNAL_TYPES": "vegas, divMacd"}

        self.assertTrue(module.should_handle_alert({"signalType": "vegas"}, env))
        self.assertTrue(module.should_handle_alert({"signalType": "DIVMACD"}, env))
        self.assertFalse(module.should_handle_alert({"signalType": "tdMd"}, env))

    def test_empty_signal_type_filter_allows_everything(self):
        module = load_module()

        self.assertTrue(module.should_handle_alert({"signalType": "tdMd"}, {}))

    def test_build_codex_args_adds_cd_before_prompt(self):
        module = load_module()

        args = module.build_codex_args("prompt text", {"CODEX_CD": r"F:\workspace\trade-agent"})

        self.assertEqual(args, ["exec", "--cd", r"F:\workspace\trade-agent", "prompt text"])


if __name__ == "__main__":
    unittest.main()
