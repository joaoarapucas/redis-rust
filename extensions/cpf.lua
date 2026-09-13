-- cpf validator

local function digits_only(text)
    if #text ~= 11 then
        return false
    end
    for i = 1, 11 do
        local character = text:sub(i, i)
        if character < "0" or character > "9" then
            return false
        end
    end
    return true
end

local function all_digits_equal(text)
    for i = 2, 11 do
        if text:sub(i, i) ~= text:sub(1, 1) then
            return false
        end
    end
    return true
end

local function calculate_check_digit(text, digit_count)
    local sum = 0
    local weight = digit_count + 1
    for i = 1, digit_count do
        local digit = tonumber(text:sub(i, i))
        sum = sum + digit * weight
        weight = weight - 1
    end
    local remainder = (sum * 10) % 11
    if remainder == 10 then
        remainder = 0
    end
    return remainder
end

local function is_valid_cpf(text)
    if all_digits_equal(text) then
        return false
    end

    local first_digit = calculate_check_digit(text, 9)
    if first_digit ~= tonumber(text:sub(10, 10)) then
        return false
    end

    local second_digit = calculate_check_digit(text, 10)
    if second_digit ~= tonumber(text:sub(11, 11)) then
        return false
    end

    return true
end

local function format_cpf(text)
    local part1 = text:sub(1, 3)
    local part2 = text:sub(4, 6)
    local part3 = text:sub(7, 9)
    local part4 = text:sub(10, 11)
    return part1 .. "." .. part2 .. "." .. part3 .. "-" .. part4
end

return {
    prefix = "cpf_",

    add = function(key, value)
        if not digits_only(value) then
            error("cpf deve conter exatamente 11 digitos numericos, sem formatacao", 0)
        end

        if not is_valid_cpf(value) then
            error("cpf invalido: digito verificador nao confere", 0)
        end

        -- uniqueness: the same cpf cant be stored under another key
        local existing_key = db_find_key(value)
        if existing_key ~= nil and existing_key ~= key then
            error("cpf ja cadastrado na chave '" .. existing_key .. "'", 0)
        end

        return nil -- keep original value (just the digits)
    end,

    get = function(key, value)
        return format_cpf(value)
    end,
}
