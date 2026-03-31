#!/usr/bin/env python3

import unittest

from scripts.tests.adversarial_runner import llm_fulfillment


class LlmFulfillmentUnitTests(unittest.TestCase):
    def test_load_llm_fulfillment_contract_accepts_repo_contracts(self):
        contract = llm_fulfillment.load_llm_fulfillment_contract()
        self.assertEqual(
            contract["schema_version"], llm_fulfillment.LLM_FULFILLMENT_PLAN_SCHEMA_VERSION
        )
        self.assertEqual(contract["frontier_action_contract_id"], "frontier_action_contract.v1")
        self.assertEqual(
            contract["container_runtime_profile_id"], "container_runtime_profile.v1"
        )
        self.assertEqual(
            contract["backend_kinds"], ["frontier_reference", "local_candidate"]
        )
        self.assertIn("browser_mode", contract["modes"])
        self.assertIn("request_mode", contract["modes"])
        self.assertEqual(contract["black_box_boundary"]["position"], "outside_attacker")
        self.assertTrue(contract["black_box_boundary"]["host_root_only_entrypoint"])
        self.assertTrue(contract["black_box_boundary"]["shuma_blind"])
        self.assertFalse(contract["black_box_boundary"]["web_search_allowed"])
        self.assertIn(
            "robots_txt", contract["black_box_boundary"]["public_host_hint_sources"]
        )
        self.assertIn(
            "shuma_repo", contract["black_box_boundary"]["forbidden_knowledge_sources"]
        )
        self.assertTrue(contract["episode_harness"]["environment_reset_required"])
        self.assertEqual(
            contract["episode_harness"]["initial_context_fields"],
            [
                "host_root_entrypoint",
                "category_objective",
                "black_box_boundary",
                "capability_envelope",
            ],
        )
        self.assertIn(
            "player_visible_protected_evidence",
            contract["episode_harness"]["allowed_memory_sources"],
        )
        self.assertIn(
            "judge_held_out_evaluation",
            contract["episode_harness"]["forbidden_memory_sources"],
        )
        self.assertFalse(contract["episode_harness"]["held_out_evaluation_visible"])

    def test_build_llm_fulfillment_plan_marks_single_provider_frontier_as_degraded(self):
        plan = llm_fulfillment.build_llm_fulfillment_plan(
            run_id="simrun-llm-fit",
            generated_tick_count=0,
            frontier_metadata={
                "provider_count": 1,
                "mode": "single_provider_self_play",
                "reduced_diversity_warning": True,
            },
            now=1_700_000_000,
        )

        self.assertEqual(plan["lane"], "bot_red_team")
        self.assertEqual(plan["tick_started_at"], 1_700_000_000)
        self.assertEqual(plan["fulfillment_mode"], "browser_mode")
        self.assertEqual(plan["backend_kind"], "frontier_reference")
        self.assertEqual(plan["backend_state"], "degraded")
        self.assertEqual(plan["backend_id"], "frontier_reference:single_provider_self_play")
        self.assertIn("local_candidate", plan["supported_backend_kinds"])
        self.assertEqual(
            plan["category_targets"],
            ["browser_agent", "agent_on_behalf_of_human"],
        )
        self.assertEqual(
            plan["capability_envelope"]["allowed_tools"],
            ["browser_navigate", "browser_snapshot", "browser_click"],
        )
        self.assertTrue(plan["capability_envelope"]["browser_automation_allowed"])
        self.assertFalse(plan["capability_envelope"]["direct_request_emission_allowed"])
        self.assertEqual(plan["black_box_boundary"]["position"], "outside_attacker")
        self.assertTrue(plan["black_box_boundary"]["public_knowledge_only"])
        self.assertFalse(plan["black_box_boundary"]["repo_visibility_allowed"])
        self.assertFalse(plan["black_box_boundary"]["judge_visibility_allowed"])
        self.assertTrue(plan["black_box_boundary"]["receipt_requirements"]["attack_trace_required"])
        self.assertEqual(
            plan["episode_harness"]["environment_reset_policy"], "fresh_episode_reset"
        )
        self.assertIn(
            "objective_completed", plan["episode_harness"]["terminal_conditions"]
        )
        self.assertEqual(plan["episode_harness"]["max_retained_episode_summaries"], 5)
        self.assertTrue(
            plan["episode_harness"]["player_visible_protected_evidence_allowed"]
        )
        self.assertFalse(plan["episode_harness"]["held_out_evaluation_visible"])

    def test_build_llm_fulfillment_plan_uses_request_mode_when_frontier_is_unavailable(self):
        plan = llm_fulfillment.build_llm_fulfillment_plan(
            run_id="simrun-llm-fit",
            generated_tick_count=1,
            frontier_metadata={
                "provider_count_configured": 0,
                "frontier_mode": "disabled",
                "reduced_diversity_warning": False,
            },
            now=1_700_000_001,
        )

        self.assertEqual(plan["fulfillment_mode"], "request_mode")
        self.assertEqual(plan["tick_started_at"], 1_700_000_001)
        self.assertEqual(plan["backend_kind"], "frontier_reference")
        self.assertEqual(plan["backend_state"], "unavailable")
        self.assertEqual(plan["backend_id"], "frontier_reference:unconfigured")
        self.assertEqual(plan["category_targets"], ["http_agent", "ai_scraper_bot"])
        self.assertEqual(
            plan["capability_envelope"]["allowed_tools"],
            ["http_get", "http_head"],
        )
        self.assertFalse(plan["capability_envelope"]["browser_automation_allowed"])
        self.assertTrue(plan["capability_envelope"]["direct_request_emission_allowed"])
        self.assertEqual(
            plan["realism_profile"]["profile_id"],
            "agentic.request_mode.v1",
        )

    def test_generate_llm_frontier_actions_request_mode_accepts_head_probe_from_provider(self):
        plan = llm_fulfillment.build_llm_fulfillment_plan(
            run_id="simrun-llm-fit",
            generated_tick_count=1,
            frontier_metadata={
                "provider_count": 1,
                "frontier_mode": "single_provider_self_play",
                "reduced_diversity_warning": False,
            },
            now=1_700_000_010,
        )
        env = {
            "SHUMA_FRONTIER_OPENAI_API_KEY": "sk-openai-test",
            "SHUMA_FRONTIER_OPENAI_MODEL": "gpt-5-mini",
        }

        result = llm_fulfillment.generate_llm_frontier_actions(
            fulfillment_plan=plan,
            host_root_entrypoint="https://example.com/",
            env_reader=lambda key: env.get(key, ""),
            provider_executor=lambda *_args, **_kwargs: {
                "actions": [
                    {
                        "action_type": "http_head",
                        "path": "/robots.txt",
                        "label": "robots_head",
                    },
                    {
                        "action_type": "http_get",
                        "path": "/research/",
                        "label": "research",
                    },
                ],
                "rationale": "Probe discoverability artifacts before focused retrieval.",
            },
        )

        self.assertEqual(result["generation_source"], "provider_response")
        self.assertEqual(
            [action["action_type"] for action in result["actions"]],
            ["http_head", "http_get"],
        )
        self.assertEqual(
            [action["method"] for action in result["actions"]],
            ["HEAD", "GET"],
        )

    def test_generate_llm_frontier_actions_uses_provider_response_when_frontier_key_exists(self):
        plan = llm_fulfillment.build_llm_fulfillment_plan(
            run_id="simrun-llm-fit",
            generated_tick_count=1,
            frontier_metadata={
                "provider_count": 1,
                "frontier_mode": "single_provider_self_play",
                "reduced_diversity_warning": False,
            },
            now=1_700_000_010,
        )
        env = {
            "SHUMA_FRONTIER_OPENAI_API_KEY": "sk-openai-test",
            "SHUMA_FRONTIER_OPENAI_MODEL": "gpt-5-mini",
        }
        observed = {}

        def fake_provider_executor(provider_spec, model_id, api_key, generation_context):
            observed["provider"] = dict(provider_spec)
            observed["model_id"] = model_id
            observed["api_key"] = api_key
            observed["context"] = generation_context
            return {
                "actions": [
                    {
                        "action_type": "http_get",
                        "path": "/robots.txt",
                        "label": "robots",
                    }
                ],
                "rationale": "Probe public crawler hints from the host only.",
            }

        result = llm_fulfillment.generate_llm_frontier_actions(
            fulfillment_plan=plan,
            host_root_entrypoint="https://example.com/",
            public_hint_paths=["/robots.txt", "/shuma/admin/config"],
            env_reader=lambda key: env.get(key, ""),
            provider_executor=fake_provider_executor,
        )

        self.assertEqual(result["generation_source"], "provider_response")
        self.assertEqual(result["provider"], "openai")
        self.assertEqual(result["model_id"], "gpt-5-mini")
        self.assertEqual(result["actions"][0]["action_type"], "http_get")
        self.assertEqual(result["actions"][0]["path"], "/robots.txt")
        self.assertEqual(observed["provider"]["provider"], "openai")
        self.assertEqual(observed["api_key"], "sk-openai-test")
        self.assertEqual(
            observed["context"]["public_hint_paths"],
            ["/robots.txt"],
        )
        self.assertEqual(
            observed["context"]["host_root_entrypoint"],
            "https://example.com/",
        )

    def test_generate_llm_frontier_actions_falls_back_when_no_provider_keys_exist(self):
        plan = llm_fulfillment.build_llm_fulfillment_plan(
            run_id="simrun-llm-fit",
            generated_tick_count=1,
            frontier_metadata={
                "provider_count_configured": 0,
                "frontier_mode": "disabled",
                "reduced_diversity_warning": False,
            },
            now=1_700_000_011,
        )

        result = llm_fulfillment.generate_llm_frontier_actions(
            fulfillment_plan=plan,
            host_root_entrypoint="https://example.com/",
            env_reader=lambda key: "",
        )

        self.assertEqual(result["generation_source"], "fallback_no_provider")
        self.assertEqual(result["provider"], "")
        self.assertEqual(result["fallback_reason"], "no_configured_frontier_provider")
        self.assertGreaterEqual(len(result["actions"]), 6)
        self.assertEqual(result["actions"][0]["action_type"], "http_get")
        self.assertEqual(result["actions"][0]["path"], "/")
        self.assertIn("http_head", [action["action_type"] for action in result["actions"]])
        self.assertIn("/robots.txt", [action["path"] for action in result["actions"]])
        self.assertIn("/sitemap.xml", [action["path"] for action in result["actions"]])
        self.assertTrue(
            any(
                path in {"/research/", "/plans/", "/work/", "/page/2/"}
                for path in [action["path"] for action in result["actions"]]
            )
        )

    def test_generate_llm_frontier_actions_falls_back_when_provider_output_breaks_mode_contract(self):
        plan = llm_fulfillment.build_llm_fulfillment_plan(
            run_id="simrun-llm-fit",
            generated_tick_count=0,
            frontier_metadata={
                "provider_count": 1,
                "frontier_mode": "single_provider_self_play",
                "reduced_diversity_warning": False,
            },
            now=1_700_000_012,
        )
        env = {
            "SHUMA_FRONTIER_OPENAI_API_KEY": "sk-openai-test",
            "SHUMA_FRONTIER_OPENAI_MODEL": "gpt-5-mini",
        }

        result = llm_fulfillment.generate_llm_frontier_actions(
            fulfillment_plan=plan,
            host_root_entrypoint="https://example.com/",
            env_reader=lambda key: env.get(key, ""),
            provider_executor=lambda *_args, **_kwargs: {
                "actions": [
                    {
                        "action_type": "http_get",
                        "path": "/",
                        "label": "wrong_mode_action",
                    }
                ]
            },
        )

        self.assertEqual(result["generation_source"], "fallback_validation_error")
        self.assertEqual(result["provider"], "openai")
        self.assertEqual(result["fallback_reason"], "provider_output_failed_validation")
        self.assertGreaterEqual(len(result["actions"]), 3)
        self.assertEqual(result["actions"][0]["action_type"], "browser_navigate")
        self.assertEqual(result["actions"][0]["path"], "/")
        self.assertTrue(
            any(
                action["path"] in {"/research/", "/plans/", "/work/", "/page/2/"}
                for action in result["actions"][1:]
            )
        )


if __name__ == "__main__":
    unittest.main()
