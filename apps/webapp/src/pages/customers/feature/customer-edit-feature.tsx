import { Link } from "@tanstack/react-router";
import { UserX } from "lucide-react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { useDirtyBaseline } from "#/hooks/use-dirty";
import { findCustomerById } from "#/pages/customers/mocks";
import type { Customer } from "#/pages/customers/types";
import { CustomerEditUI } from "#/pages/customers/ui/customer-edit-ui";

interface CustomerEditFeatureProps {
	customerId: string;
}

export function CustomerEditFeature({ customerId }: CustomerEditFeatureProps) {
	const customer = findCustomerById(customerId);

	if (!customer) {
		return (
			<div className="flex flex-col items-center justify-center gap-3 p-12 text-center">
				<div className="flex size-14 items-center justify-center rounded-2xl border bg-card">
					<UserX className="size-6 text-muted-foreground" />
				</div>
				<div>
					<p className="font-semibold">Client introuvable</p>
					<p className="text-sm text-muted-foreground">
						Aucun client ne correspond à cet identifiant.
					</p>
				</div>
				<Button asChild variant="outline" className="rounded-xl">
					<Link to="/customers">Retour aux clients</Link>
				</Button>
			</div>
		);
	}

	return <CustomerEditInner customer={customer} />;
}

function CustomerEditInner({ customer }: { customer: Customer }) {
	const [form, setForm] = useState<Customer>(customer);
	const [isSaving, setIsSaving] = useState(false);

	const { isDirty, changedKeys, commit, reset } = useDirtyBaseline(
		customer,
		form,
	);

	const handleChange = (patch: Partial<Customer>) => {
		setForm((prev) => ({ ...prev, ...patch }));
	};

	const handleAddressChange = (patch: Partial<Customer["address"]>) => {
		setForm((prev) => ({ ...prev, address: { ...prev.address, ...patch } }));
	};

	const handleReset = () => {
		setForm(customer);
		reset();
	};

	const handleSave = async () => {
		setIsSaving(true);
		try {
			await new Promise((r) => setTimeout(r, 800));
			console.log("[customers] saved", form);
			commit(form);
		} finally {
			setIsSaving(false);
		}
	};

	return (
		<CustomerEditUI
			customer={customer}
			form={form}
			isDirty={isDirty}
			changedKeys={changedKeys}
			isSaving={isSaving}
			onChange={handleChange}
			onAddressChange={handleAddressChange}
			onReset={handleReset}
			onSave={handleSave}
		/>
	);
}
