'use client';

import React, { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { api, ContractSearchParams, Contract } from '@/lib/api';
import ContractCard from '@/components/ContractCard';
import { Search, Filter, Package, ArrowUpDown } from 'lucide-react';
import { useSearchParams, useRouter } from 'next/navigation';

export function ContractsContent() {
  const router = useRouter();
  const searchParams = useSearchParams();

  const [filters, setFilters] = useState<ContractSearchParams>({
    query: searchParams.get('query') || '',
    network: (searchParams.get('network') as ContractSearchParams['network']) || undefined,
    verified_only: searchParams.get('verified_only') === 'true',
    sort_by: (searchParams.get('sort_by') as ContractSearchParams['sort_by']) || (searchParams.get('query') ? 'relevance' : 'created_at'),
    sort_order: (searchParams.get('sort_order') as ContractSearchParams['sort_order']) || 'desc',
    page: parseInt(searchParams.get('page') || '1'),
    page_size: 12,
  });

  // Sync state with URL
  useEffect(() => {
    const params = new URLSearchParams();
    if (filters.query) params.set('query', filters.query);
    if (filters.network) params.set('network', filters.network);
    if (filters.verified_only) params.set('verified_only', 'true');
    if (filters.sort_by) params.set('sort_by', filters.sort_by);
    if (filters.sort_order) params.set('sort_order', filters.sort_order);
    if (filters.page && filters.page > 1) params.set('page', filters.page.toString());

    router.push(`?${params.toString()}`, { scroll: false });
  }, [filters, router]);

  const { data, isLoading } = useQuery({
    queryKey: ['contracts', filters],
    queryFn: () => api.getContracts(filters),
  });

  const handleSearch = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setFilters(prev => ({ ...prev, query: prev.query, page: 1 }));
  };

  return (
    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-4xl font-bold text-gray-900 dark:text-white mb-2">
          Browse Contracts
        </h1>
        <p className="text-gray-600 dark:text-gray-400">
          Discover verified Soroban smart contracts on the Stellar network
        </p>
      </div>

      {/* Search and Filters */}
      <div className="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 p-6 mb-8">
        <form onSubmit={handleSearch} className="mb-4">
          <div className="relative">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
            <input
              type="text"
              value={filters.query || ''}
              onChange={(e) => setFilters({ ...filters, query: e.target.value })}
              placeholder="Search contracts..."
              className="w-full pl-12 pr-4 py-3 rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>
        </form>

        <div className="flex flex-wrap gap-4">
          <div className="flex items-center gap-2">
            <Filter className="w-4 h-4 text-gray-500" />
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Filters:
            </span>
          </div>

          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={filters.verified_only || false}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setFilters(prev => ({ ...prev, verified_only: e.target.checked, page: 1 }))}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="text-sm text-gray-700 dark:text-gray-300">
              Verified only
            </span>
          </label>

          <select
            value={filters.network || ''}
            onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setFilters(prev => ({ ...prev, network: e.target.value as ContractSearchParams['network'], page: 1 }))}
            className="px-3 py-1 rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-sm text-gray-700 dark:text-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="">All Networks</option>
            <option value="mainnet">Mainnet</option>
            <option value="testnet">Testnet</option>
            <option value="futurenet">Futurenet</option>
          </select>

          <div className="h-6 w-px bg-gray-200 dark:bg-gray-800 self-center hidden sm:block" />

          <div className="flex items-center gap-2">
            <ArrowUpDown className="w-4 h-4 text-gray-500" />
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Sort:
            </span>
          </div>

          <select
            value={filters.sort_by || ''}
            onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setFilters(prev => ({ ...prev, sort_by: e.target.value as ContractSearchParams['sort_by'], page: 1 }))}
            className="px-3 py-1 rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-sm text-gray-700 dark:text-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="created_at">Newest First</option>
            <option value="updated_at">Recently Updated</option>
            <option value="popularity">Most Popular</option>
            <option value="deployments">Most Deployed</option>
            <option value="interactions">Most Interactions</option>
            {filters.query && <option value="relevance">Relevance</option>}
          </select>

          <select
            value={filters.sort_order || 'desc'}
            onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setFilters(prev => ({ ...prev, sort_order: e.target.value as ContractSearchParams['sort_order'], page: 1 }))}
            className="px-3 py-1 rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-sm text-gray-700 dark:text-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="desc">Descending</option>
            <option value="asc">Ascending</option>
          </select>
        </div>
      </div>

      {/* Results */}
      {isLoading ? (
        <div className="text-center py-12">
          <div className="inline-block w-8 h-8 border-4 border-blue-600 border-t-transparent rounded-full animate-spin" />
        </div>
      ) : data && data.items.length > 0 ? (
        <>
          <div className="mb-4 text-sm text-gray-600 dark:text-gray-400">
            Showing {data.items.length} of {data.total} contracts
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8">
            {data.items.map((contract: Contract) => (
              <ContractCard key={contract.id} contract={contract} />
            ))}
          </div>

          {/* Pagination */}
          {data.total_pages > 1 && (
            <div className="flex items-center justify-center gap-2">
              <button
                onClick={() => setFilters({ ...filters, page: (filters.page || 1) - 1 })}
                disabled={(filters.page || 1) <= 1}
                className="px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-700 text-gray-700 dark:text-gray-300 disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
              >
                Previous
              </button>

              <span className="text-sm text-gray-600 dark:text-gray-400">
                Page {filters.page || 1} of {data.total_pages}
              </span>

              <button
                onClick={() => setFilters({ ...filters, page: (filters.page || 1) + 1 })}
                disabled={(filters.page || 1) >= data.total_pages}
                className="px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-700 text-gray-700 dark:text-gray-300 disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
              >
                Next
              </button>
            </div>
          )}
        </>
      ) : (
        <div className="text-center py-12 bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800">
          <Package className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <p className="text-gray-600 dark:text-gray-400">No contracts found</p>
        </div>
      )}
    </div>
  );
}
