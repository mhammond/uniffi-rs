from callbacks import *

# Simple example just to see it work.
# Pass in a string, get a string back.
# Pass in nothing, get unit back.
class OnCallAnsweredImpl(OnCallAnswered):
    def __init__(self):
        self.yes_count = 0
        self.busy_count = 0
        self.string_received = ""
        self.last_sim = None

    def hello(self, sim):
        print("got hello on sim", sim.name())
        self.last_sim = sim
        self.yes_count += 1
        return f"Hi hi {self.yes_count}"

    def busy(self, sim):
        self.last_sim = sim
        self.busy_count += 1

    def text_received(self, sim, text):
        self.last_sim = sim
        self.string_received = text

sim = get_sim_cards()[0]
cb_object = OnCallAnsweredImpl()
telephone = Telephone()

telephone.call(sim, domestic=True, call_responder=cb_object)
assert cb_object.busy_count == 0, f"yes_count={cb_object.busy_count} (should be 0)"
assert cb_object.yes_count == 1, f"yes_count={cb_object.yes_count} (should be 1)"

telephone.call(sim, domestic=True, call_responder=cb_object)
assert cb_object.busy_count == 0, f"yes_count={cb_object.busy_count} (should be 0)"
assert cb_object.yes_count == 2, f"yes_count={cb_object.yes_count} (should be 2)"

telephone.call(sim, domestic=False, call_responder=cb_object)
assert cb_object.busy_count == 1, f"yes_count={cb_object.busy_count} (should be 1)"
assert cb_object.yes_count == 2, f"yes_count={cb_object.yes_count} (should be 2)"
assert cb_object.string_received != "", f"string_received='{cb_object.string_received}' (should be a message)"

cb_object2 = OnCallAnsweredImpl()
telephone.call(sim, domestic=True, call_responder=cb_object2)
assert cb_object2.busy_count == 0, f"yes_count={cb_object2.busy_count} (should be 0)"
assert cb_object2.yes_count == 1, f"yes_count={cb_object2.yes_count} (should be 1)"

print("The last call was made on the SIM card", cb_object2.last_sim.name())
cb_object2.last_sim = None
print("Rust thinks the last call was made on the SIM card", telephone.get_last_sim().name())

# Now demo foreign implemented traits
class PythonSimCard(SimCardTrait):
    def name(self):
        return "python"

sim = rt_sim(PythonSimCard())
print("Round-tripped sim with name", sim.name())

telephone.call(sim, domestic=True, call_responder=cb_object2)

print("The last call was made on the SIM card", cb_object2.last_sim.name())
# uncomment this to panic on a todo for freeing!
#   cb_object2.last_sim = None
