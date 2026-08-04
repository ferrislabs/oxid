import { useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { MOCK_CUSTOMERS } from '#/pages/customers/mocks'
import type { Customer } from '#/pages/customers/types'
import { CustomerListUI } from '#/pages/customers/ui/customer-list-ui'

export function CustomerListFeature() {
	const navigate = useNavigate()
	const [customers] = useState<Customer[]>(MOCK_CUSTOMERS)
	const [isLoading] = useState(false)

	// Not implemented: these screens run on mocks, and the handlers logged the
	// customer record to the console, which shipped to production.
	const handleAdd = () => {}

	const handleEdit = (customer: Customer) => {
		void navigate({
			to: '/customers/$customerId',
			params: { customerId: customer.id },
		})
	}

	const handleDelete = (_customer: Customer) => {}

	return (
		<CustomerListUI
			customers={customers}
			isLoading={isLoading}
			onAdd={handleAdd}
			onEdit={handleEdit}
			onDelete={handleDelete}
		/>
	)
}
