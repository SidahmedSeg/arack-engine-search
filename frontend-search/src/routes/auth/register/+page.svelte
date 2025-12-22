<script lang="ts">
import { goto } from '$app/navigation';
import { Button } from '$lib/components/ui/button';
import { Input } from '$lib/components/ui/input';
import { Label } from '$lib/components/ui/label';
import * as Card from '$lib/components/ui/card';
import { api } from '$lib/stores/api';
import { initRegistrationFlow, submitRegistration } from '$lib/api/kratos';
import { onMount } from 'svelte';

// Step tracking
let currentStep = $state(1);
const totalSteps = 3;

// Step 1: Personal Information
let firstName = $state('');
let lastName = $state('');
let dateOfBirth = $state('');
let gender = $state('');

// Step 2: Username Selection
let selectedUsername = $state('');
let customUsername = $state('');
let suggestions = $state<Array<{username: string; email: string; available: boolean}>>([]);
let checkingCustom = $state(false);
let customAvailable = $state<boolean | null>(null);

// Step 3: Password
let password = $state('');
let confirmPassword = $state('');

// Common
let error = $state('');
let loading = $state(false);
let flowId = $state('');

// Password strength calculation
const passwordStrength = $derived.by(() => {
    if (password.length === 0) return 0;
    let strength = 0;
    if (password.length >= 8) strength++;
    if (password.length >= 12) strength++;
    if (/[a-z]/.test(password) && /[A-Z]/.test(password)) strength++;
    if (/\d/.test(password)) strength++;
    if (/[^a-zA-Z0-9]/.test(password)) strength++;
    return Math.min(4, strength);
});

const strengthLabel = $derived.by(() => {
    switch(passwordStrength) {
        case 1: return 'Weak';
        case 2: return 'Fair';
        case 3: return 'Good';
        case 4: return 'Strong';
        default: return '';
    }
});

const strengthColor = $derived.by(() => {
    switch(passwordStrength) {
        case 1: return 'bg-red-500';
        case 2: return 'bg-orange-500';
        case 3: return 'bg-yellow-500';
        case 4: return 'bg-green-500';
        default: return 'bg-gray-200';
    }
});

// Form validation
const step1Valid = $derived(
    firstName.trim() !== '' &&
    lastName.trim() !== '' &&
    dateOfBirth !== '' &&
    gender !== ''
);

const step2Valid = $derived(
    selectedUsername !== '' ||
    (customUsername !== '' && customAvailable === true)
);

const step3Valid = $derived(
    password.length >= 8 &&
    password === confirmPassword &&
    passwordStrength >= 2
);

onMount(async () => {
	const flow = await initRegistrationFlow();
	if (flow) {
		flowId = flow.id;
	} else {
		error = 'Failed to initialize registration';
	}
});

// Load username suggestions when moving to step 2
async function loadSuggestions() {
    if (!step1Valid) return;

    loading = true;
    error = '';

    try {
        const result = await api.suggestUsernames({
            first_name: firstName,
            last_name: lastName
        });
        suggestions = result.suggestions;

        // Auto-select first available
        const firstAvailable = suggestions.find(s => s.available);
        if (firstAvailable) {
            selectedUsername = firstAvailable.username;
        }

        currentStep = 2;
    } catch (err: any) {
        error = err.message || 'Failed to load suggestions';
    } finally {
        loading = false;
    }
}

// Check custom username availability (debounced)
let debounceTimer: any;
function checkCustomUsername() {
    clearTimeout(debounceTimer);

    if (customUsername.length < 3) {
        customAvailable = null;
        return;
    }

    checkingCustom = true;

    debounceTimer = setTimeout(async () => {
        try {
            const result = await api.checkUsername({ username: customUsername });
            customAvailable = result.available;
        } catch (error) {
            customAvailable = false;
        } finally {
            checkingCustom = false;
        }
    }, 500);
}

$effect(() => {
    if (customUsername) {
        checkCustomUsername();
    }
});

// Submit registration to Kratos
async function submitRegistration_() {
    if (!flowId) {
        error = 'Registration flow not initialized';
        return;
    }

    if (!step3Valid) {
        error = 'Please complete all fields';
        return;
    }

    loading = true;
    error = '';

    try {
        const finalUsername = selectedUsername || customUsername;
        const email = `${finalUsername}@arack.io`;

        await submitRegistration(flowId, {
            email: email,
            password: password,
            first_name: firstName,
            last_name: lastName,
            username: finalUsername,
            date_of_birth: dateOfBirth,
            gender: gender
        });

        // Success - redirect to login
        goto('/auth/login');
    } catch (err: any) {
        error = err.message || 'Registration failed';
    } finally {
        loading = false;
    }
}
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4">
    <Card.Root class="w-full max-w-md">
        <Card.Header>
            <Card.Title>Create your Arack account</Card.Title>
            <Card.Description>
                Step {currentStep} of {totalSteps}
            </Card.Description>
        </Card.Header>

        <Card.Content>
            <!-- Progress Bar -->
            <div class="mb-6">
                <div class="flex gap-2">
                    {#each Array(totalSteps) as _, i}
                        <div
                            class="h-2 flex-1 rounded-full transition-colors {i < currentStep ? 'bg-blue-600' : 'bg-gray-200'}"
                        ></div>
                    {/each}
                </div>
            </div>

            {#if error}
                <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded-md text-red-600 text-sm">
                    {error}
                </div>
            {/if}

            {#if currentStep === 1}
                <!-- Step 1: Personal Information -->
                <div class="space-y-4">
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <Label for="firstName">First Name</Label>
                            <Input
                                id="firstName"
                                bind:value={firstName}
                                placeholder="John"
                                required
                                disabled={loading}
                            />
                        </div>
                        <div>
                            <Label for="lastName">Last Name</Label>
                            <Input
                                id="lastName"
                                bind:value={lastName}
                                placeholder="Doe"
                                required
                                disabled={loading}
                            />
                        </div>
                    </div>

                    <div>
                        <Label for="dateOfBirth">Date of Birth</Label>
                        <Input
                            id="dateOfBirth"
                            type="date"
                            bind:value={dateOfBirth}
                            max={new Date().toISOString().split('T')[0]}
                            required
                            disabled={loading}
                        />
                    </div>

                    <div>
                        <Label>Gender</Label>
                        <div class="flex gap-4 mt-2">
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input
                                    type="radio"
                                    bind:group={gender}
                                    value="male"
                                    class="w-4 h-4"
                                    disabled={loading}
                                />
                                <span>Male</span>
                            </label>
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input
                                    type="radio"
                                    bind:group={gender}
                                    value="female"
                                    class="w-4 h-4"
                                    disabled={loading}
                                />
                                <span>Female</span>
                            </label>
                        </div>
                    </div>

                    <Button
                        onclick={loadSuggestions}
                        disabled={!step1Valid || loading}
                        class="w-full"
                    >
                        {loading ? 'Loading...' : 'Continue'}
                    </Button>
                </div>

            {:else if currentStep === 2}
                <!-- Step 2: Username Selection -->
                <div class="space-y-4">
                    <div>
                        <Label>Choose your email address</Label>
                        <div class="space-y-2 mt-2">
                            {#each suggestions.filter(s => s.available).slice(0, 2) as suggestion}
                                <label
                                    class="flex items-center gap-3 p-3 border rounded-lg cursor-pointer hover:bg-gray-50 transition-colors {selectedUsername === suggestion.username ? 'border-blue-600 bg-blue-50' : 'border-gray-200'}"
                                >
                                    <input
                                        type="radio"
                                        bind:group={selectedUsername}
                                        value={suggestion.username}
                                        onclick={() => customUsername = ''}
                                        class="w-4 h-4"
                                    />
                                    <div class="flex-1">
                                        <div class="font-medium">{suggestion.email}</div>
                                        {#if selectedUsername === suggestion.username}
                                            <div class="text-xs text-green-600">✓ Selected</div>
                                        {/if}
                                    </div>
                                </label>
                            {/each}
                        </div>
                    </div>

                    <div class="relative">
                        <Label>Or create custom username</Label>
                        <div class="flex items-center gap-2 mt-2">
                            <div class="relative flex-1">
                                <Input
                                    bind:value={customUsername}
                                    placeholder="custom_username"
                                    class="pr-24"
                                    oninput={() => {
                                        selectedUsername = '';
                                        checkCustomUsername();
                                    }}
                                />
                                <span class="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 text-sm">
                                    @arack.io
                                </span>
                            </div>
                        </div>
                        {#if checkingCustom}
                            <p class="text-sm text-gray-500 mt-1">
                                Checking availability...
                            </p>
                        {:else if customUsername.length >= 3}
                            {#if customAvailable}
                                <p class="text-sm text-green-600 mt-1">
                                    ✓ Available!
                                </p>
                            {:else}
                                <p class="text-sm text-red-600 mt-1">
                                    ✗ Already taken
                                </p>
                            {/if}
                        {/if}
                    </div>

                    <div class="flex gap-2">
                        <Button
                            variant="outline"
                            onclick={() => currentStep = 1}
                            class="flex-1"
                            disabled={loading}
                        >
                            Back
                        </Button>
                        <Button
                            onclick={() => currentStep = 3}
                            disabled={!step2Valid || loading}
                            class="flex-1"
                        >
                            Continue
                        </Button>
                    </div>
                </div>

            {:else if currentStep === 3}
                <!-- Step 3: Password -->
                <div class="space-y-4">
                    <div>
                        <Label for="password">Password</Label>
                        <Input
                            id="password"
                            type="password"
                            bind:value={password}
                            placeholder="Minimum 8 characters"
                            required
                            disabled={loading}
                        />
                        {#if password.length > 0}
                            <div class="mt-2 flex gap-1">
                                {#each Array(4) as _, i}
                                    <div
                                        class="h-2 flex-1 rounded-full transition-colors {i < passwordStrength ? strengthColor : 'bg-gray-200'}"
                                    ></div>
                                {/each}
                            </div>
                            <p class="text-xs text-gray-500 mt-1">
                                {strengthLabel}
                            </p>
                        {/if}
                    </div>

                    <div>
                        <Label for="confirmPassword">Confirm Password</Label>
                        <Input
                            id="confirmPassword"
                            type="password"
                            bind:value={confirmPassword}
                            placeholder="Re-enter password"
                            required
                            disabled={loading}
                        />
                        {#if confirmPassword.length > 0}
                            {#if password === confirmPassword}
                                <p class="text-sm text-green-600 mt-1">✓ Passwords match</p>
                            {:else}
                                <p class="text-sm text-red-600 mt-1">✗ Passwords don't match</p>
                            {/if}
                        {/if}
                    </div>

                    <div class="flex gap-2">
                        <Button
                            variant="outline"
                            onclick={() => currentStep = 2}
                            class="flex-1"
                            disabled={loading}
                        >
                            Back
                        </Button>
                        <Button
                            onclick={submitRegistration_}
                            disabled={!step3Valid || loading}
                            class="flex-1"
                        >
                            {loading ? 'Creating Account...' : 'Create Account'}
                        </Button>
                    </div>
                </div>
            {/if}
        </Card.Content>

        <Card.Footer>
            <p class="text-sm text-gray-500 text-center w-full">
                Already have an account?
                <a href="/auth/login" class="text-blue-600 hover:underline">
                    Sign in
                </a>
            </p>
        </Card.Footer>
    </Card.Root>
</div>
