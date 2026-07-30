import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, expect, test } from "vitest";
import ControlsHarness from "./ControlsHarness.test.svelte";

afterEach(cleanup);

test("shared controls expose names and keyboard-operated state", async () => {
  render(ControlsHarness);

  const checkbox = screen.getByRole("checkbox", { name: "Восстановление" });
  await fireEvent.click(checkbox);
  expect(checkbox.getAttribute("aria-checked")).toBe("true");

  const select = screen.getByRole("button", { name: "Назначение" });
  await fireEvent.keyDown(select, { key: "ArrowDown" });
  await fireEvent.keyDown(select, { key: "Enter" });
  expect(screen.getByText("Выбрано: recall")).toBeTruthy();
});

test("dialog traps an accessible title and closes with Escape", async () => {
  render(ControlsHarness);
  await fireEvent.click(screen.getByRole("button", { name: "Открыть справку" }));
  expect(await screen.findByRole("dialog", { name: "Справка" })).toBeTruthy();
  await fireEvent.keyDown(document.activeElement ?? document.body, { key: "Escape" });
  expect(screen.queryByRole("dialog", { name: "Справка" })).toBeNull();
});

test("icon action requires one accessible name and provides a tooltip", async () => {
  render(ControlsHarness);
  const action = screen.getByRole("button", { name: "Удалить" });
  action.focus();
  expect((await screen.findByRole("tooltip")).textContent).toContain("Удалить");
});
